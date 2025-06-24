## rocket_emulator

由 https://github.com/chipsalliance/rocket-chip 自定义配置的MaxExtension RV64生成

```scala
class MaxExtensionRV64Config extends Config(
  new WithB ++           // Bitmanip扩展
  new WithFP16 ++        // 半精度浮点扩展
  new WithHypervisor ++  // 虚拟化扩展
  new DefaultConfig      // 基础配置（RV64IMAFDC + 1个大核心）
)

// RV32版本的最大扩展配置
class MaxExtensionRV32Config extends Config(
  new WithB ++           // Bitmanip扩展
  new WithFP16 ++        // 半精度浮点扩展
  new WithHypervisor ++  // 虚拟化扩展
  new WithRV32 ++        // 32位RISC-V
  new WithNBigCores(1) ++ 
  new WithCoherentBusTopology ++ 
  new BaseConfig
)
```