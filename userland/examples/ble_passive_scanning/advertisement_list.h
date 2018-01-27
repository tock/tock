// this is left outside the class because it doesn't work
// code-generation bug?!
const int MAX_SIZE = 10;

struct AdvertisementList {
  int currSize;
  std::forward_list<Advertisement> list;

  AdvertisementList()
      : currSize(0)
  {
    list.resize(MAX_SIZE);
  }

  bool add(const Advertisement& advertisement)
  {
    if (currSize < MAX_SIZE) {
      list.push_front(advertisement);
      currSize += 1;
      return true;
    } else {
      return false;
    }
  }

  bool containsDevice(const Advertisement& advertisement)
  {
    for (auto it = list.begin(); it != list.end(); ++it) {
      if (it->device_detected(advertisement)) {
        return true;
      }
    }
    return false;
  }

  bool newData(const Advertisement& advertisement)
  {
    for (auto it = list.begin(); it != list.end(); ++it) {
      if (*it != advertisement) {
         list.remove(*it);
         list.push_front(advertisement);
         return true;
      }
    }
    return false;
  }

  void printList()
  {
    printf("--------------------------LIST-------------------------\r\n\r\n");
    for (auto it = list.begin(); it != list.end(); ++it) {
      it->print();
    }
    printf("--------------------------END---------------------------\r\n\r\n");
  }
};
