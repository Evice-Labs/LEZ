fn main() {
    // If it finds packages that depend on "risc0-zkvm" (such as Demo A and B),
    // it will automatically assemble them into Static ELF with the perfect ZKVM Linker Script.
    risc0_build::embed_methods();
}