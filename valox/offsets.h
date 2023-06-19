// this is the offsets file that autogenerates the classes used in the hack, can use both c style and dumps.host style offsets declared in any order (besides bitfields)
// this will ignore classes if you forget the ; and have to declare pointers in hex in the comment, you can change this in memory_macros/src/cparser_lib.rs

struct UActorComponent : UObject {
};

struct UStreamableRenderAsset : UObject {
};

struct USkinnedAsset : UStreamableRenderAsset {
};

struct USceneComponent : UActorComponent {
	struct FVector RelativeLocation; // 0x164(0xc)
	struct FRotator RelativeRotation; // 0x170(0xc)
	struct FVector RelativeScale3D; // 0x17c(0xc)
	struct FVector ComponentVelocity; // 0x188(0xc)
    
	struct FTransform ComponentToWorld; // 0x250
};

struct UPrimitiveComponent : USceneComponent {
};

struct UMeshComponent : UPrimitiveComponent {
};

struct USkinnedMeshComponent : UMeshComponent {
	struct USkeletalMesh* SkeletalMesh; // 0x598(0x08)

	struct FBoxSphereBounds CachedWorldSpaceBounds; // 0x73c(0x1c)

	uint8_t bRecentlyRenderedPad : 6; // 0x71f(0x1)
	/// True if mesh has been recently rendered, false otherwise
	uint8_t bRecentlyRendered : 1; // 0x71f(0x1)
};

struct USkeleton : UObject {
};

struct USkeletalMesh : UStreamableRenderAsset {
	struct USkeleton* Skeleton; // 0x88(0x08)
};

struct UWorld : UObject {
	struct ULevel* PersistentLevel; // 0x38(0x08)
	struct TArray<struct ULevel*> Levels; // 0x158(0x10)
	struct UGameInstance* OwningGameInstance; // 0x1a0(0x08)
	struct AGameStateBase* GameState; // 0x140(0x08)
};

struct AActor : UObject {
	uint8_t bLocallyHidden : 1; // 0x60
	struct USceneComponent* RootComponent; // 0x230(0x08)
};

struct AInfo : AActor {
};

struct APlayerState : AInfo {
	struct APawn* PawnPrivate; // 0x438(0x08)
	struct FString PlayerNamePrivate; // 0x4b8(0x10)
};

struct AGameStateBase : AInfo {
	struct TArray<struct APlayerState*> PlayerArray; // 0x3e8(0x10)
};

struct ULevel : UObject {
    struct TArray<struct AActor*> Actors; // 0xa0
};

struct UGameInstance : UObject {
	struct TArray<struct ULocalPlayer*> LocalPlayers; // 0x40(0x10)
};

struct UPlayer : UObject {
	struct APlayerController* PlayerController; // 0x38(0x08)
};

struct ULocalPlayer : UPlayer {
};

struct AController : AActor {
	struct APlayerState* PlayerState; // 0x3d8(0x08)
	struct APawn* Pawn; // 0x408(0x08)
	struct ACharacter* Character; // 0x418(0x08)
	struct FRotator ControlRotation; // 0x440(0x0c)
};

struct APlayerController : AController {
	struct APawn* AcknowledgedPawn; // 0x460(0x08)

	struct APlayerCameraManager* PlayerCameraManager; // 0x478(0x08)
};

struct APlayerCameraManager : AActor {
	struct FCameraCacheEntry CameraCachePrivate; // 0x2020(0x6a0)
};

struct APawn : AActor {
	struct APlayerState* PlayerState; // 0x3f0(0x08)
	struct AController* Controller; // 0x408(0x08)
};

struct ACharacter : APawn {
	struct USkeletalMeshComponent* Mesh; // 0x430(0x08)
};

struct USkeletalMeshComponent : USkinnedMeshComponent {
	struct BoneArray BoneArray; // 0x5c8
};


struct AShooterCharacter : ACharacter {
	struct UDamageableComponent* DamageHandler; // 0xa10(0x08)
	struct UAresInventory* Inventory; // 0x9b0(0x8)
	struct uint32_t CachedFresnelColor; // 0x6e0(0x4)
};

struct UDamageableComponent : UActorComponent {
	struct TArray<struct UDamageSectionComponent*> DamageSections; // 0x198(0x10)
};

struct UDamageSectionComponent : UActorComponent {
	DamageSectionType DamageType; // 0x128(0x01)
	float Life; // 0x134(0x04)
	float MaximumLife; // 0x138(0x04)
};

struct AAresPlayerStateBase : APlayerState {
	struct UBaseTeamComponent* TeamComponent; // 0x628(0x08)
};

struct UBaseTeamComponent : UActorComponent {
	char teamId; // 0xF8;
};

struct AShooterPlayerState : AAresPlayerStateBase {
	struct AShooterCharacter* SpawnedCharacter; // 0x920(0x8)
};


struct UAresInventory : UActorComponent {
	struct AAresEquippable* CurrentEquippable; // 0x248(0x08)
};

struct AAresItem : AActor {

};

struct AAresEquippable : AAresItem {
	struct USkeletalMeshComponent* Mesh1P; // 0xcc8(0x08)
};

struct AGameObject : AActor {
	struct UDamageableComponent* DamageHandler; // 0x3f0(0x8)
};

struct AAresOnGroundEquippable : AGameObject { 
	struct AAresEquippable* MyEquippable; // 0x408(0x8)
};

struct AEquippableGroundPickup_C : AAresOnGroundEquippable { 
	struct FText DisplayName; // 0x4a0(0x18)
	bool isBomb : 1; // 0x4b8(0x1)
};

struct APlantedBomb_C : AGameObject {
	bool BombHasExploded : 1; // 0x4b0(0x1)
	float BombExplodeOuterRadius; // 0x4cc(0x4)
	float BombExplodeInnerRadius; // 0x4d0(0x4)
	float TimeRemainingToExplode; // 0x4d4(0x4)
	bool BombHasBeenDefused : 1; // 0x4dd(0x1)
	float DefuseProgress; // 0x500(0x4)
	bool BombDefuseState; // 0x4e9(0x1)
};

struct ATimedBomb_C : APlantedBomb_C {
};

struct AGameObject_Wushu_4_SmokeZone_C : AGameObject {
};

struct AGameObject_Thorne_E_Wall_Segment_Fortifying_C : AGameObject {
};

struct ABasePawn_C : AShooterCharacter {

};

struct ABasePlayerCharacter_C : ABasePawn_C {

};

struct ATrainingBot_PC_C : ABasePlayerCharacter_C {

};