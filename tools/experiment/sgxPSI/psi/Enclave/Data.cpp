#include <string>
#include "stdlib.h"

struct Data {
    uint64_t value1;
    uint64_t value2;

    bool operator==(const Data& d) const
    {
        return value1 == d.value1 && value2 == d.value2;
    }

    size_t hash() const {
        std::string str = std::to_string(value1) + "_" + std::to_string(value2);
        std::hash<std::string> h;
        return h(str);
    }
};

struct Hasher {
    size_t operator()(const Data& k) const {
        std::string str = std::to_string(k.value1) + "_" + std::to_string(k.value2);
        std::hash<std::string> h;
        return h(str);
    }
};