#include <cstdint>
#include <iostream>

extern "C" std::uint64_t aic_main(std::uint64_t input);
extern "C" int aic_probe(std::uint64_t input, std::uint64_t* result);

int main() {
    std::uint64_t input = 0;
    while (std::cin >> input) {
        std::uint64_t result = 0;
        if (aic_probe(input, &result) != 1) {
            return 2;
        }
        std::cout << result << '\n';
    }
    return std::cin.eof() ? 0 : 1;
}
