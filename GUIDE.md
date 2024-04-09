# guide
- dummy the asset you want to hook (create actor with same folder structure in your unreal project)
<details>
<summary>example</summary>

![](https://i.imgur.com/FN06HLY.png)
![](https://i.imgur.com/Cmm1CbI.png)
</details>

- delete all events (you hook events using functions)
<details>
<summary>example</summary>

![](https://i.imgur.com/osSLR0P.png)
</details>

- i recommend using [ue4ss](https://docs.ue4ss.com/installation-guide.html) to [dump cxx headers](https://docs.ue4ss.com/feature-overview/dumpers.html#c-header-generator) to reference when dummying functions
<details>
<summary>example</summary>

```cpp
#ifndef UE4SS_SDK_BP_RestChair_HPP
#define UE4SS_SDK_BP_RestChair_HPP

class ABP_RestChair_C : public AActor
{
    FPointerToUberGraphFrame UberGraphFrame;                                          // 0x0290 (size: 0x8)
    class UWidgetComponent* Widget;                                                   // 0x0298 (size: 0x8)
    class USphereComponent* examineSphere;                                            // 0x02A0 (size: 0x8)
    class UArrowComponent* startPosArrow;                                             // 0x02A8 (size: 0x8)
    class USphereComponent* Sphere;                                                   // 0x02B0 (size: 0x8)
    class UStaticMeshComponent* Cylinder_80E0306B;                                    // 0x02B8 (size: 0x8)
    class USceneComponent* DefaultSceneRoot;                                          // 0x02C0 (size: 0x8)
    bool healing?;                                                                    // 0x02C8 (size: 0x1)
    bool displayingPrompt;                                                            // 0x02C9 (size: 0x1)
    bool overlappingPlayer?;                                                          // 0x02CA (size: 0x1)
    class UUI_ExaminePrompt_C* As UI Examine Prompt;                                  // 0x02D0 (size: 0x8)

    void BPI_InteractConfirm(int32 Response, class AActor* interactionTarget);
    void ReceiveBeginPlay();
    void BPI_TryInteract(class AActor* interactee);
    void BndEvt__BP_RestChair_Sphere1_K2Node_ComponentBoundEvent_0_ComponentBeginOverlapSignature__DelegateSignature(class UPrimitiveComponent* OverlappedComponent, class AActor* OtherActor, class UPrimitiveComponent* OtherComp, int32 OtherBodyIndex, bool bFromSweep, const FHitResult& SweepResult);
    void BndEvt__BP_RestChair_Sphere1_K2Node_ComponentBoundEvent_2_ComponentEndOverlapSignature__DelegateSignature(class UPrimitiveComponent* OverlappedComponent, class AActor* OtherActor, class UPrimitiveComponent* OtherComp, int32 OtherBodyIndex);
    void BPI_EndInteract();
    void ExecuteUbergraph_BP_RestChair(int32 EntryPoint);
}; // Size: 0x2D8

#endif
```
</details>

- dummy the function you want to hook with same arguments and return type and put an `hook_` before the name
<details>
<summary>example</summary>

![](https://i.imgur.com/eMeIgAi.png)
</details>

- to call the original duplicate your dummied function and replace `hook_` with `orig_`
<details>
<summary>example</summary>

![](https://i.imgur.com/FzQ9EE2.png)
</details>

- code whatever you want (you can dummy other functions and use them as normal)
<details>
<summary>example</summary>

![](https://i.imgur.com/e8jwjj1.png)
![](https://i.imgur.com/J4IvrIE.png)
</details>

- cook/package the project
<details>
<summary>example</summary>

![](https://i.imgur.com/qj4ng9t.png)

</details>

- unpack the original asset from the game 
- if your game uses paks use [umodel](https://www.gildor.org/en/projects/umodel) or [fmodel](https://fmodel.app/) to do this
- if your game uses iostore use [zentools-ue4](https://github.com/WistfulHopes/ZenTools-UE4/releases) or [zentools](https://github.com/Archengius/ZenTools/releases) to do this
- apply hooks [using spaghetti](README.md#usage)
<details>
<summary>example</summary>

```
spaghetti "...\pseudoregalia\Saved\Cooked\Windows\pseudoregalia\Content\Blueprints\LevelActors\BP_RestChair.uasset" "...\UModelSaved\Game\Blueprints\LevelActors\BP_RestChair.uasset" -o "...\raw\hook_p\pseudoregalia\Content\Blueprints\LevelActors\BP_RestChair.uasset" -v 5.1
BPI_TryInteract hooked
```
</details>

- pack your hooked blueprint into a mod
- if your game uses paks use [unrealpak](http://fluffyquack.com/tools/unrealpak.rar) or [repak](https://github.com/trumank/repak/releases)
- if your game uses iostore use [zentools-ue4](https://github.com/WistfulHopes/ZenTools-UE4/releases) or [zentools](https://github.com/Archengius/ZenTools/releases) to do this
- the function should be hooked in-game! easy as pie (hopefully)
<details>
<summary>example</summary>

interacting with the chair now kills you!

![](https://i.imgur.com/LAceEnJ.gif)
</details>
<!---
- the default events have different names internally you will need to use

| Event           | Internal Name    |
|-----------------|------------------|
| Event BeginPlay | ReceiveBeginPlay |
| Event End Play  | ReceiveEndPlay   |
| Event Tick      | ReceiveTick      |
-->
