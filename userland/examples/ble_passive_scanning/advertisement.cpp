#include "advertisement.h"

Advertisement::Advertisement() : advertisementType_(0), len_(0) {
  std::memset(&address_, 0, ADDRESS_SIZE);
  std::memset(&data_, 0, DATA_SIZE);
}

Advertisement::Advertisement(const unsigned char* buf, int len) : advertisementType_(buf[ADV_TYPE]), len_(len)
{
  std::memcpy(&address_, &buf[2], ADDRESS_SIZE);
  if (len_ > DATA_SIZE) {
    std::memcpy(&data_, &buf[8], len_ - DATA_SIZE);
  }
}

bool Advertisement::device_detected(const Advertisement& other) const
{
  return std::memcmp(&address_, &other.address_, ADDRESS_SIZE) == 0;
}

bool Advertisement::operator==(const Advertisement& other) const {
  return std::memcmp(this, &other, sizeof(Advertisement)) == 0;
}

bool Advertisement::operator!=(const Advertisement& other) const {
  return std::memcmp(this, &other, sizeof(Advertisement)) != 0;
}

void Advertisement::print() const
{
  printf("BLE Address: %02x %02x %02x %02x %02x %02x\r\n", address_[5],
         address_[4], address_[3], address_[2], address_[1], address_[0]);
  printf("BLE AD_Type: %s\r\n", advertisementTypeToStr());
  printf("Packet Size (address + data): %d\r\n", len_);
  printf("Data: ");
  for (int i = 0; i < len_ - ADDRESS_SIZE; i++) {
    printf("%02x ", data_[i]);
  }
  printf("\r\n\r\n");
}

const char* Advertisement::advertisementTypeToStr() const
{
  switch (advertisementType_) {
    case 0:
      return "ADV_IND";
    case 1:
      return "ADV_DIRECT_IND";
    case 2:
      return "NON_CONNECT_IND";
    case 3:
      return "SCAN_REQ";
    case 4:
      return "SCAN_RSP";
    case 5:
      return "CONNECT_REQ";
    case 6:
      return "ADV_SCAN_IND";
    default:
      return "INVALID ADVERTISEMENT TYPE";
  }
}

bool Advertisement::validAdvertisement(const unsigned char* buf, int len)
{
  if (buf != nullptr && len >= ADDRESS_SIZE && buf[0] <= 6) {
    return true;
  } else {
    return false;
  }
}

