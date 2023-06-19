
use std::collections::HashMap;
use crate::*;

// this is relative to your working directory
include_c_file!("offsets.h");

#[c_enum]
pub enum DamageSectionType {
    Health = 0,
    Shield = 1,
    Temporary = 2,
}

#[derive(Debug, Default, Clone, Copy)]
#[allow(dead_code)]
pub struct HpValue {
    pub value: f32,
    pub max: f32,
}
#[derive(Debug, Default, Clone, Copy)]
pub struct HpTypes {
    pub hp: HpValue,
    pub sheild: HpValue,
    pub temp: HpValue,
}

lazy_static::lazy_static! {
    pub static ref GUN_PAIRS : HashMap::<String, &'static str> = vec![
    ("Ability_Melee_Base_C", "TacticalKnife"),
    ("BasePistol_C", "Classic"),
    ("SawedOffShotgun_C", "Shorty"),
    ("AutomaticPistol_C", "Frenzy"),
    ("LugerPistol_C", "Ghost"),
    ("RevolverPistol_C", "Sheriff"),
    ("Vector_C", "Stinger"),
    ("SubMachineGun_MP5_C", "Spectre"),
    ("AutomaticShotgun_C", "Judge"),
    ("PumpShotgun_C", "Bucky"),
    ("AssaultRifle_Burst_C", "Bulldog"),
    ("DMR_C", "Guardian"),
    ("AssaultRifle_ACR_C", "Phantom"),
    ("AssaultRifle_AK_C", "Vandal"),
    ("LeverSniperRifle_C", "Marshal"),
    ("BoltSniper_C", "Operator"),
    ("LightMachineGun_C", "Ares"),
    ("HeavyMachineGun_C", "Odin")].into_iter().map(|(a,b)|  (a.to_owned(),b)).collect();
}
