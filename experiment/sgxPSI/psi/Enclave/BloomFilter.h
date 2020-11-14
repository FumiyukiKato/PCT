#include <vector>
#include "Data.cpp"

class BloomFilter {
public:
  BloomFilter(uint64_t size, uint8_t numHashes);
 
  void add(const Data *data);
  bool possiblyContains(const Data *data) const;
  float prob(float num);
 
private:
  uint8_t m_numHashes;
  std::vector<bool> m_bits;
};