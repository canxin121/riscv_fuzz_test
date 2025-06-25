## rocket_emulator

Generated from https://github.com/chipsalliance/rocket-chip, commit f517abbf41abb65cea37421d3559f9739efd00a9,with custom MaxExtension RV64 configuration

```scala
class MaxExtensionRV64Config extends Config(
  new WithB ++           // Bitmanip extension
  new WithFP16 ++        // Half-precision floating-point extension
  new WithHypervisor ++  // Hypervisor extension
  new DefaultConfig      // Base configuration (RV64IMAFDC + 1 big core)
)

// RV32 version of maximum extension configuration
class MaxExtensionRV32Config extends Config(
  new WithB ++           // Bitmanip extension
  new WithFP16 ++        // Half-precision floating-point extension
  new WithHypervisor ++  // Hypervisor extension
  new WithRV32 ++        // 32-bit RISC-V
  new WithNBigCores(1) ++ 
  new WithCoherentBusTopology ++ 
  new BaseConfig
)
``` 