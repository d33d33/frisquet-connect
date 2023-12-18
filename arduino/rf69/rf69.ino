#include <SPI.h>
#include <RH_RF69.h>

// from RH_RF69.cpp
#define CONFIG_FSK (RH_RF69_DATAMODUL_DATAMODE_PACKET | RH_RF69_DATAMODUL_MODULATIONTYPE_FSK | RH_RF69_DATAMODUL_MODULATIONSHAPING_FSK_NONE)
#define CONFIG_WHITE (RH_RF69_PACKETCONFIG1_PACKETFORMAT_VARIABLE | RH_RF69_PACKETCONFIG1_DCFREE_NONE | RH_RF69_PACKETCONFIG1_CRC_ON | RH_RF69_PACKETCONFIG1_CRCAUTOCLEAROFF | RH_RF69_PACKETCONFIG1_ADDRESSFILTERING_NONE)


/************ Radio Setup ***************/
// First 3 here are boards w/radio BUILT-IN. Boards using FeatherWing follow.
#if defined (__AVR_ATmega32U4__)  // Feather 32u4 w/Radio
  #define RFM69_CS    8
  #define RFM69_INT   7
  #define RFM69_RST   4
  #define LED        13

#elif defined(ADAFRUIT_FEATHER_M0) || defined(ADAFRUIT_FEATHER_M0_EXPRESS) || defined(ARDUINO_SAMD_FEATHER_M0)  // Feather M0 w/Radio
  #define RFM69_CS    8
  #define RFM69_INT   3
  #define RFM69_RST   4
  #define LED        13

#elif defined(ARDUINO_ADAFRUIT_FEATHER_RP2040_RFM)  // Feather RP2040 w/Radio
  #define RFM69_CS   16
  #define RFM69_INT  21
  #define RFM69_RST  17
  #define LED        LED_BUILTIN

#elif defined (__AVR_ATmega328P__)  // Feather 328P w/wing
  #define RFM69_CS    4  //
  #define RFM69_INT   3  //
  #define RFM69_RST   2  // "A"
  #define LED        13

#elif defined(ESP8266)  // ESP8266 feather w/wing
  #define RFM69_CS    2  // "E"
  #define RFM69_INT  15  // "B"
  #define RFM69_RST  16  // "D"
  #define LED         0

#elif defined(ARDUINO_ADAFRUIT_FEATHER_ESP32S2) || defined(ARDUINO_NRF52840_FEATHER) || defined(ARDUINO_NRF52840_FEATHER_SENSE)
  #define RFM69_CS   10  // "B"
  #define RFM69_INT   9  // "A"
  #define RFM69_RST  11  // "C"
  #define LED        13

#elif defined(ESP32)  // ESP32 feather w/wing
  #define RFM69_CS   33  // "B"
  #define RFM69_INT  27  // "A"
  #define RFM69_RST  13  // same as LED
  #define LED        13

#elif defined(ARDUINO_NRF52832_FEATHER)  // nRF52832 feather w/wing
  #define RFM69_CS   11  // "B"
  #define RFM69_INT  31  // "C"
  #define RFM69_RST   7  // "A"
  #define LED        17

#endif

/* Teensy 3.x w/wing
#define RFM69_CS     10  // "B"
#define RFM69_INT     4  // "C"
#define RFM69_RST     9  // "A"
#define RFM69_IRQN   digitalPinToInterrupt(RFM69_INT)
*/

/* WICED Feather w/wing
#define RFM69_CS     PB4  // "B"
#define RFM69_INT    PA15 // "C"
#define RFM69_RST    PA4  // "A"
#define RFM69_IRQN   RFM69_INT
*/

// Singleton instance of the radio driver
RH_RF69 rf69(RFM69_CS, RFM69_INT);

void setup() {
  Serial.begin(115200);
  while (!Serial) delay(1); // Wait for Serial Console (comment out line if no computer)

  pinMode(LED, OUTPUT);
  pinMode(RFM69_RST, OUTPUT);
  digitalWrite(RFM69_RST, LOW);

  Serial.println("Feather RFM69!");
  Serial.println();

  // manual reset
  digitalWrite(RFM69_RST, HIGH);
  delay(10);
  digitalWrite(RFM69_RST, LOW);
  delay(10);

  if (!rf69.init()) {
    Serial.println("RFM69 radio init failed");
    while (1);
  }
  Serial.println("RFM69 radio init OK!");
  if (!rf69.setFrequency(868.96)) {
    Serial.println("setFrequency failed");
    while (1);
  }

  // If you are using a high power RF69 eg RFM69HW, you *must* set a Tx power with the
  // ishighpowermodule flag set like this:
  rf69.setTxPower(20, true);
  
  // BITRATE: FXOSC(32_000_000) / 25_000 = 1280 => 0x500
  // FDEV: 50_000 / FSTEP(61) = 820 => 0x334
  // DccFreq 0b010
  // RxBwMant 0b10
  // RxBwExp 1
  // RXBW: DccFreq RxBwMant RxBwExp

  const RH_RF69::ModemConfig config{CONFIG_FSK, 0x05, 0x00, 0x03, 0x34, 0b01010001, 0b01010001, CONFIG_WHITE};
  rf69.setModemRegisters(&config);
  rf69.setPreambleLength(4);

  rf69.setPromiscuous(true);

  uint8_t syncwords[] = { 0xff, 0xff, 0xff, 0xff }; // pairing
  rf69.setSyncWords(syncwords, sizeof(syncwords));
}

int now = millis();

void loop()
{
  if (rf69.available()) {
    // Should be a message for us now   
    uint8_t buf[RH_RF69_MAX_MESSAGE_LEN];
    uint8_t len = sizeof(buf);
    if (rf69.recv(buf, &len)) {
      if (!len) return;
      buf[len] = 0;

      Serial.print("Received [");
      Serial.print(len);
      Serial.print("] ");
      Serial.print("FROM: 0x");
      printHex(rf69.headerFrom ());
      Serial.print(" | TO: 0x");
      printHex(rf69.headerTo ());
      Serial.print(" | ID: 0x");
      printHex(rf69.headerId ());
      Serial.print(" | FLAGS: 0x");
      printHex(rf69.headerFlags ());
      Serial.print(" | RSSI: ");
      Serial.println(rf69.lastRssi(), DEC);

      printHex(len+4);
      printHex(rf69.headerTo ());
      printHex(rf69.headerFrom ());
      printHex(rf69.headerId ());
      printHex(rf69.headerFlags ());
      for (int i = 0; i < len; i++) {
        printHex(buf[i]);
      }
      Serial.println();
      
      digitalWrite(LED, HIGH);
    } else {
      Serial.println("Receive failed");
    }
  }

  if (millis() - now > 1000) {
      digitalWrite(LED, LOW);
//    Serial.println(rf69.maxMessageLength());
    Serial.print("echo");
    Serial.println(now);
    now = millis();
  }

  if (Serial.available() > 0) {
    // read the incoming byte:
    int incomingByte = 0; // for incoming serial data
    incomingByte = Serial.read();

    // say what you got:
    Serial.print("Received: ");
    Serial.println(incomingByte, DEC);
  }
}

void printHex(byte n) {
  Serial.print(n < 16 ? "0" : "");
  Serial.print(n, HEX);
}
