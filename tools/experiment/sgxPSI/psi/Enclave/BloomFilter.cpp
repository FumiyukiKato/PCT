#include "BloomFilter.h"
#include "MurmurHash3.h"

#include <array>
#include <math.h>
 
BloomFilter::BloomFilter(uint64_t size, uint8_t numHashes)
      : m_bits(size),
        m_numHashes(numHashes) {}


std::array<uint64_t, 2> hash(const Data *data) {
  std::array<uint64_t, 2> hashValue;
  uint64_t hash_of_data[2] = {data->value1, data->value2};
  MurmurHash3_x64_128(hash_of_data, sizeof(uint64_t)*2, 0, hashValue.data());
 
  return hashValue;
}

inline uint64_t nthHash(uint8_t n,
                        uint64_t hashA,
                        uint64_t hashB,
                        uint64_t filterSize) {
    return (hashA + n * hashB) % filterSize;
}

void BloomFilter::add(const Data *data) {
  auto hashValues = hash(data);
 
  for (int n = 0; n < m_numHashes; n++) {
      m_bits[nthHash(n, hashValues[0], hashValues[1], m_bits.size())] = true;
  }
}
 
bool BloomFilter::possiblyContains(const Data *data) const {
  auto hashValues = hash(data);
 
  for (int n = 0; n < m_numHashes; n++) {
      if (!m_bits[nthHash(n, hashValues[0], hashValues[1], m_bits.size())]) {
          return false;
      }
  }
 
  return true;
}

float BloomFilter::prob(float num) {
  return powf(1.0 - powf(1.0 - 1.0 / this->m_bits.size(), this->m_numHashes * num), this->m_numHashes);
}