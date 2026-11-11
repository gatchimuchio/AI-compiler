#include <cstdint>
#include <iostream>

extern "C" std::uint64_t aic_main(std::uint64_t input);
extern "C" int aic_probe(std::uint64_t input, std::uint64_t* result);

int main() {
    std::uint64_t input = 0;
    while (std::cin >> input) {
        std::uint64_t result = 0;
        std::uint16_t x87_before = 0;
        std::uint32_t mxcsr_before = 0;
        asm volatile("fnstcw %0\n\tstmxcsr %1"
                     : "=m"(x87_before), "=m"(mxcsr_before) : : "memory");
        const int probe_status = aic_probe(input, &result);
        std::uint64_t returned_flags = 0;
        std::uint16_t x87_after = 0;
        std::uint32_t mxcsr_after = 0;
        // The test consumer is built with -mno-red-zone for this flags snapshot.
        asm volatile("pushfq\n\tpopq %0\n\tcld\n\tfnstcw %1\n\tstmxcsr %2"
                     : "=&r"(returned_flags), "=m"(x87_after), "=m"(mxcsr_after)
                     : : "memory", "cc");
        if ((returned_flags & 0x400) != 0 || x87_before != x87_after ||
            ((mxcsr_before ^ mxcsr_after) & 0xffc0) != 0) {
            asm volatile("fldcw %0\n\tldmxcsr %1"
                         : : "m"(x87_before), "m"(mxcsr_before) : "memory");
            return 3;
        }
        if (probe_status != 1) {
            return 2;
        }
        std::cout << result << '\n';
    }
    return std::cin.eof() ? 0 : 1;
}
