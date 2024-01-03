#include <Arduino.h>
#include <heltec.h>
#include <RadioLib.h>

#include <vector>
#include <iostream>

SX1262 radio = new Module(SS, DIO0, RST_LoRa, BUSY_LoRa);
int receiveddCount = 0;

typedef std::vector<uint8_t> ByteBuffer;
typedef std::string ByteString;
boolean sleeping = false;

int ascii2val(char c)
{
    int iRetVal;

    if ((c >= '0') && (c <= '9'))
    {
        iRetVal = (c - '0');
    }
    else if ((c >= 'a') && (c <= 'f'))
    {
        iRetVal = (c - 'a' + 10);
    }
    else if ((c >= 'A') && (c <= 'F'))
    {
        iRetVal = (c - 'A' + 10);
    }
    else
    {
        iRetVal = 0;
    }

    return iRetVal;
}

ByteBuffer unhexlify(const std::string &InBuffer)
{
    ByteBuffer OutBuffer(InBuffer.size() / 2 + 1);

    for (size_t i = 0, j = 0; i < InBuffer.size(); i += 2, ++j)
    {
        uint8_t *dest = &OutBuffer[j];
        *dest++ = (((ascii2val(InBuffer[i]) << 4) | (ascii2val(InBuffer[i + 1]))));
    }

    return OutBuffer;
}

void setup()
{

    Heltec.begin(true /*DisplayEnable Enable*/, false /*LoRa Disable*/, true /*Serial Enable*/);
    // Heltec.display->init();
    Heltec.display->flipScreenVertically();
    // Heltec.display->setFont(ArialMT_Plain_10);

    Serial.begin(115200);
    int state = radio.beginFSK();
    state = radio.setFrequency(868.96);
    state = radio.setBitRate(25.0);
    state = radio.setFrequencyDeviation(50.0);
    state = radio.setRxBandwidth(250.0);
    state = radio.setPreambleLength(4);
    // uint8_t network_id[] = {5, 218, 46, 226};
    uint8_t network_id[] = {0x12, 0x34, 0x56, 0x78};
    state = radio.setSyncWord(network_id, sizeof(network_id));
}

String byteArrayToHexString(uint8_t *byteArray, int length)
{
    String result = "";
    for (int i = 0; i < length; i++)
    {
        char hex[3];
        sprintf(hex, "%02X", byteArray[i]);
        result += hex;
    }
    return result;
}

void updateDisplay(int len, byte *buf, int rssi)
{
    Heltec.display->clear();
    Heltec.display->drawString(0, 0, "RECEIVED length: " + String(len));
    Heltec.display->drawString(0, 11, "total: " + String(receiveddCount) + "|RSSI: " + String(rssi) + "dBm");
    Heltec.display->drawString(0, 22, byteArrayToHexString(buf, len));

    Heltec.display->display();
}

void loop()
{
    byte byteArr[RADIOLIB_SX126X_MAX_PACKET_LENGTH];
    if (!sleeping)
    {
        int state = radio.receive(byteArr, 0);
        if (state == RADIOLIB_ERR_NONE)
        {
            int len = radio.getPacketLength();
            int rssi = radio.getRSSI();
            Serial.printf("%02X", len, rssi);
            for (int i = 0; i < len; i++)
                Serial.printf("%02X", byteArr[i]);
            Serial.println("");
            receiveddCount++;
            updateDisplay(len, byteArr, rssi);
        }
    }

    if (Serial.available())
    {

        String serialData = Serial.readStringUntil('\n');
        // Serial.println("Received command: " + serialData);

        if (serialData.startsWith("SLP:"))
        {
            // radio.sleep();
            // Serial.println("Identify SLP command");
            sleeping = true;
        }
        else if (serialData.startsWith("LST:"))
        {

            // Serial.println("Identify LST command");
            sleeping = false;
            radio.standby();
        }
        else
        {
            String payload = serialData.substring(5);
            if (sleeping == true)
            {
                radio.standby();
            }
            if (serialData.startsWith("CMD: "))
            {

                // Serial.println("Identify CMD command : " + payload);
                ByteBuffer byteData = unhexlify(payload.c_str());

                uint8_t len = byteData[0];
                if (len > 63)
                {
                    // Serial.println("ERR: CMD: Failed, can't be more than 63 bytes");
                }
                else
                {
                    radio.transmit(&byteData[1], byteData[0], 0);
                }
            }
            if (serialData.startsWith("NID: "))
            {
                String network_id = payload;
                // Serial.println("Identify NID command, will set " + network_id);
                ByteBuffer networkIdBB = unhexlify(network_id.c_str());
                radio.setSyncWord(&networkIdBB[0], networkIdBB.size() - 1);

                // radio.setSyncWord(byteArr + 5, 8);
            }
            if (sleeping == true)
            {
                radio.sleep();
            }
        }
    }
}
