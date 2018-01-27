// this is left outside the class because it doesn't work
// code-generation bug?!
const int ADDRESS_SIZE = 6;
const int DATA_SIZE = 32;
const int DATA_START = 8;
const int DATA_END = 39;
const int ADV_TYPE = 0;


class Advertisement {
  private:
    unsigned char advertisementType;
    int len;
    unsigned char address[ADDRESS_SIZE];
    unsigned char data[DATA_SIZE];

    const char* advertisementTypeToStr(unsigned char type)
    {
      switch (type) {
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

  public:
    Advertisement(const unsigned char* b, int l)
    {
      advertisementType = b[ADV_TYPE];
      len = l;
      std::memcpy(&address, &b[2], ADDRESS_SIZE);
      if (len > DATA_SIZE) {
        std::memcpy(&data, &b[8], len - DATA_SIZE);
      }
    }

    Advertisement()
      : advertisementType(0)
        , len(0)
  {
  }

    bool device_detected(const Advertisement& other)
    {
      return std::memcmp(&address, &other.address, ADDRESS_SIZE) == 0;
    }

    bool operator==(const Advertisement& other) {
      return std::memcmp(this, &other, sizeof(Advertisement)) == 0;
    }

    bool operator!=(const Advertisement& other) {
      return std::memcmp(this, &other, sizeof(Advertisement)) != 0;
    }

    void print()
    {
      printf("BLE Address: %02x %02x %02x %02x %02x %02x\r\n", address[5],
          address[4], address[3], address[2], address[1], address[0]);
      printf("BLE AD_Type: %s\r\n",
          Advertisement::advertisementTypeToStr(advertisementType));
      printf("Packet Size (address + data): %d\r\n", len);
      printf("Data: ");
      for (int i = 0; i < len - ADDRESS_SIZE; i++) {
        printf("%02x ", data[i]);
      }
      printf("\r\n\r\n");
    }

    static bool validAdvertisement(const unsigned char* buf, int len)
    {
      if (buf != nullptr && len >= ADDRESS_SIZE && buf[0] <= 6) {
        return true;
      } else {
        return false;
      }
    }
};
