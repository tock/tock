#pragma once

#include <cstring>

// this is left outside the class because it doesn't work
// code-generation bug?!
const int ADDRESS_SIZE = 6;
const int DATA_SIZE = 32;
const int DATA_START = 8;
const int DATA_END = 39;
const int ADV_TYPE = 0;

class Advertisement {
  private:
    unsigned char advertisementType_;
    int len_;
    unsigned char address_[ADDRESS_SIZE];
    unsigned char data_[DATA_SIZE];

  public:
    // Constructors 
    Advertisement(const unsigned char* buf, int len);
    Advertisement();
    
    // Methods
    bool device_detected(const Advertisement& other) const;
    bool operator==(const Advertisement& other) const;
    bool operator!=(const Advertisement& other) const;
    void print() const;
    const char* advertisementTypeToStr() const;
    static bool validAdvertisement(const unsigned char* buf, int len);
};
