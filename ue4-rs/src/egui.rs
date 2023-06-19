use egui_overlay::egui::epaint::QuadraticBezierShape;
use egui_overlay::egui::*;

use crate::classes::Camera;
use crate::primatives::{FMinimalViewInfo, FReal, FRotator, FTransform, FVector, FVector2D};

pub struct EguiCamera<'a> {
    pub camera: Camera,
    pub world_location: FVector,
    pub rotation: FRotator,
    pub forwards: FVector,
    pub right: FVector,
    pub up: FVector,
    pub screen_center: Pos2,
    pub screen_size: Pos2,

    pub layer: &'a RenderLayer<'a>,
}

pub struct RenderLayer<'a> {
    pub ctx: &'a Context,
    pub layer: LayerId,
    pub painter: Painter,
    pub width: f32,
    pub height: f32,
    //pub image_cache: &'a HashMap<u64, TextureHandle>,
    //pub config: &'a RenderConfig,
}

impl<'a> RenderLayer<'a> {
    pub fn new(
        ctx: &'a Context,
        //image_cache: &'a HashMap<u64, TextureHandle>,
        //config: &'a RenderConfig,
    ) -> Self {
        let width = ctx.screen_rect().width();
        let height = ctx.screen_rect().height();

        let layer = LayerId::background();
        Self {
            ctx,
            layer,
            painter: ctx.layer_painter(layer),
            width,
            height,
            //config,
            //image_cache,
        }
    }

    pub fn new_camera_minimal(&self, minimal_info: FMinimalViewInfo) -> EguiCamera {
        EguiCamera::from_minimal(minimal_info, &self)
    }
}

impl<'a> EguiCamera<'a> {
    pub fn from_minimal(info: FMinimalViewInfo, layer: &'a RenderLayer) -> Self {
        let camera = Camera::new(
            info,
            FVector2D::new(layer.width as FReal, layer.height as FReal),
        );

        Self {
            camera,
            world_location: info.location,
            rotation: info.rotation,
            screen_center: Pos2::new(layer.width * 0.5, layer.height * 0.5),
            screen_size: Pos2::new(layer.width, layer.height),
            layer,
            forwards: info.rotation.to_direction(),
            right: FRotator::new(
                info.rotation.pitch,
                info.rotation.yaw + 90.,
                info.rotation.roll,
            )
            .to_direction(),
            up: FRotator::new(
                info.rotation.pitch + 90.,
                info.rotation.yaw,
                info.rotation.roll,
            )
            .to_direction(),
        }
    }

    pub fn from_minimal_fix(mut info: FMinimalViewInfo, layer: &'a RenderLayer) -> Self {
        let rad_fov: FReal =
            ((layer.height * 0.5) / (0.0174532925 * (info.fov * 0.5)).tan()) as FReal;
        info.fov = 2.0 * 57.2957795 * (((layer.width * 0.5) / rad_fov).atan());
        Self::from_minimal(info, layer)
    }

    pub fn distance(&self, world_position: FVector) -> FReal {
        self.world_location.distance(world_position)
    }

    pub fn draw_outline_text(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
    ) {
        self.draw_outline_text_color(pos, anchor, text, font_id, Color32::WHITE);
    }

    pub fn draw_outline_text_color(
        &self,
        pos: Pos2,
        anchor: Align2,
        text: impl ToString,
        font_id: FontId,
        color: Color32,
    ) {
        let text = text.to_string();
        const SIZE: f32 = 1.0;
        for offset_x in [-SIZE, 0., SIZE] {
            for offset_y in [-SIZE, 0., SIZE] {
                self.painter().text(
                    pos + Vec2::new(offset_x, offset_y),
                    anchor,
                    &text,
                    font_id.clone(),
                    Color32::BLACK,
                );
            }
        }
        self.painter().text(pos, anchor, text, font_id, color);
    }

    // in case you want to switch renderer, you can just do a egui compliant layer
    pub fn painter(&self) -> &'a Painter {
        &self.layer.painter
    }

    pub fn w2s(&self, world_position: FVector) -> Pos2 {
        let v2 = self.camera.w2s(world_position);
        Pos2::new(v2.x as f32, v2.y as f32)
    }

    pub fn w2sc(&self, world_position: FVector) -> Option<Pos2> {
        let v2 = self.camera.w2sc(world_position)?;
        Some(Pos2::new(v2.x as f32, v2.y as f32))
    }

    pub fn draw_tracers(&self, world_position: FVector, color: Color32) {
        let other = self.w2s(world_position);
        const NORMAL: f32 = 90.0;
        if other.distance(self.screen_center) < NORMAL {
            return;
        }

        self.painter().line_segment(
            [
                self.screen_center + (other - self.screen_center).normalized() * NORMAL,
                other,
            ],
            Stroke::new(1., color),
        );
    }

    #[inline]
    pub fn draw_line(&self, a: Pos2, b: Pos2, stroke: Stroke) {
        self.painter().line_segment([a, b], stroke)
    }

    pub fn draw_bezier_points(
        &self,
        points: &mut Vec<Pos2>,
        radius: f32,
        stroke: Stroke,
        filled: bool,
    ) {
        if filled {
            if let (Some(&first), Some(&last)) = (points.first(), points.last()) {
                points.insert(0, last);
                points.push(first);
            }
        } else {
            // too lazy to write it correctly, so we do some extra mem move
            if let Some(&first) = points.first() {
                points.insert(0, first);
            }
            if let Some(&last) = points.last() {
                points.push(last);
            }
        }

        for &[a, b, c] in points.array_windows::<3>() {
            let ab = b - a;
            let bc = c - b;
            let start = a + (ab.normalized() * radius.min(ab.length()) * 0.5);
            let end = b + (bc.normalized() * radius.min(bc.length()) * 0.5);
            self.draw_bezier_curve(start, b, end, stroke);
        }
    }

    pub fn draw_bezier_curve(&self, a: Pos2, b: Pos2, c: Pos2, stroke: Stroke) {
        const MIN_DIST: f32 = 0.1;
        // this is actually required because bezier drawing panics if 2 points are very close
        if a.distance_sq(b) < MIN_DIST {
            self.draw_line(a, c, stroke);
        } else if a.distance_sq(c) < MIN_DIST {
            self.draw_line(a, b, stroke);
        } else if b.distance_sq(c) < MIN_DIST {
            self.draw_line(a, b, stroke);
        } else {
            self.painter().add(Shape::QuadraticBezier(
                QuadraticBezierShape::from_points_stroke(
                    [a, b, c],
                    false,
                    Color32::TRANSPARENT,
                    stroke,
                ),
            ));
        }
    }

    pub fn draw_2d_box(&self, pos: Pos2, width: f32, height: f32, fill: Color32, outline: Stroke) {
        let rect = Rect::from_center_size(pos, Vec2::new(width, height));
        self.painter().rect_filled(rect, 0., fill);
        self.painter().rect_stroke(rect, 0., outline);
    }

    pub fn draw_skeleton<const INNER: usize, const OUTER: usize>(
        &self,
        c2w: FTransform,
        bone_data: &Vec<FTransform>,
        skeleton: &[[usize; INNER]; OUTER],
        skeleton_mode: SkeletonRenderMode,
    ) {
        if bone_data.is_empty() {
            return;
        }

        let mut items: Vec<Vec<Pos2>> = skeleton
            .iter()
            .map(|line| {
                line.iter()
                    .map(|&idx| self.w2sc(c2w.get_bone_with_rotation(&bone_data[idx])))
                    .filter_map(|x| x)
                    .collect()
            })
            .filter(|x: &Vec<_>| !x.is_empty())
            .collect();

        match skeleton_mode {
            SkeletonRenderMode::Normal { stroke } => {
                for screen_bones in &items {
                    for &line in screen_bones.array_windows::<2>() {
                        self.draw_line(line[0], line[1], stroke);
                    }
                }
            }
            SkeletonRenderMode::Bezier { stroke, radius } => {
                let radius = radius / self.distance(c2w.translation) as f32;
                for screen_bones in &mut items {
                    self.draw_bezier_points(screen_bones, radius, stroke, false)
                }
            }
        }
    }
}

pub enum SkeletonRenderMode {
    /// Normal Skelly
    Normal { stroke: Stroke },
    /// Rounded skeleton, radius
    Bezier { stroke: Stroke, radius: f32 },
}
