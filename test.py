#!/usr/bin/env python3

import sys
import serial
import time


def cmd(c):
    time.sleep(1)
    print('>', c)
    s.write(f'{c}\r\n'.encode())
    time.sleep(1)
    res = b''
    while True:
        r = s.read(4)
        if len(r) == 0:
            res = res.decode('ascii', errors='ignore')
            print(res)
            return res
        res += r


# SIM800 Series AT Command Manual V1.09
# https://www.elecrow.com/wiki/images/2/20/SIM800_Series_AT_Command_Manual_V1.09.pdf

# SIM800 Series TCP/IP Application Note
# https://www.waveshare.com/w/upload/2/25/SIM800_Series_TCPIP_Application_Note_V1.03.pdf

def send_data(addr, port, data):
    # AT+CREG?: wait for network registration status
    #    0 Not registered, MT is not currently searching a new operator to register to
    #    1 Registered, home network
    #    2 Not registered, but MT is currently searching a new operator to register to
    #    3 Registration denied
    #    4 Unknown
    #    5 Registered, roaming
    while True:
        res = cmd('AT+CREG?')
        # done if 1
        # wait if 2
        # reset module if 0,3,4,5
        if res.find('+CREG: 0,1') >= 0:
            break
        elif res.find('+CREG: 0,2') >= 0:
            time.sleep(2)
        else:
            return None

    # AT+CGATT=1: attach to GPRS service
    while True:
        res = cmd('AT+CGATT=1')
        if res.find('ERROR') >= 0:
            time.sleep(2)
        elif res.find('OK') == -1:
            time.sleep(2)
        else:
            break

    # AT+CIPMODE=1: set transparent mode
    while True:
        res = cmd('AT+CIPMODE=1')
        if res.find('ERROR') >= 0:
            cmd('AT+CIPSTATUS')
            time.sleep(2)
        elif res.find('OK') == -1:
            time.sleep(2)
        else:
            break

    # TODO: AT+CIPCCFG: configure transfer options (see section 3.2.2)

    # AT+CSTT="3g.ge": set APN
    while True:
        res = cmd('AT+CSTT="3g.ge"')
        if res.find('ERROR') >= 0:
            time.sleep(2)
        elif res.find('OK') == -1:
            time.sleep(2)
        else:
            break

    cmd('AT+CIICR')
    # check ERROR/OK
    cmd('AT+CIFSR') # this one is important
    # check ERROR/OK

    # AT+CIPSTART="TCP","116.228.221.51","8500": start connection
    while True:
        res = cmd(f'AT+CIPSTART="TCP","{addr}","{port}"')
        ## NB! returns "OK\r\nCONNECT" instead of "CONNECT OK"
        # "CONNECT FAIL"
        if res.find('CONNECT FAIL'):
            time.sleep(2)
        elif res.find('CONNECT') >= 0:
            break
        else:
            time.sleep(2)

    s.write(f'{data}\r\n'.encode())
    time.sleep(1)
    s.write(f'+++'.encode())
    time.sleep(1)
    cmd('AT+CIPCLOSE')
    cmd('AT+CIPSHUT')
    return True


def send_text_data(addr, port, data):
    # AT+CGATT=1: attach to GPRS service
    while True:
        res = cmd('AT+CGATT=1')
        if res.find('ERROR') >= 0:
            time.sleep(2)
        elif res.find('OK') == -1:
            time.sleep(2)
        else:
            break

    # AT+CSTT="3g.ge": set APN
    while True:
        res = cmd('AT+CSTT="3g.ge"')
        if res.find('ERROR') >= 0:
            time.sleep(2)
        elif res.find('OK') == -1:
            time.sleep(2)
        else:
            break

    cmd('AT+CIICR')
    cmd('AT+CIFSR') # this one is important

    # AT+CIPSTART="TCP","116.228.221.51","8500": start connection
    while True:
        res = cmd(f'AT+CIPSTART="TCP","{addr}","{port}"')
        if res.find('CONNECT OK') >= 0:
            break
        else:
            time.sleep(2)

    cmd('AT+CIPSEND')
    time.sleep(1)
    s.write(f'{data}\r\n\x1a'.encode())
    time.sleep(1)
    cmd('AT+CIPCLOSE')
    cmd('AT+CIPSHUT')
    return True


# FIXME: maybe with higher verbosity level it will be easier to parse
# responses?

# TODO: use AT+CIPSTATUS to check current IP status in case of command error or timeout


# TODO: "AT+CNMI=2,1,0,0,0" or "AT+CNMI=2,0,0,0,0"  to drop incoming SMS

# FIXME: we can loose signal at any point


# Quit data mode by pulling DTR or using “+++”
# ATO: return to data mode
# NB. It is better to open the hardware flow control for using transparent mode, by setting AT+IFC=2,2.


# If TCP/UDP connection exists, the DCD pin will
# be active (low). At any time if the connection is dropped, DCD pin will go inactive (high).

# If an error occurs in TCP/UDP connection, for example TCP sending data error or TCP connection dropping,
# it is suggested to close the connection by command AT+CIPCLOSE and then restart the connection by
# AT+CIPSTART. If the error still occurs, please use AT+CIPSHUT to shut off the PDP context and then
# restart the connection. If these two methods above can’t help to solve it, SIMCom recommends user to
# reset the module.



# FIXME: explicit port config
# set timeout in seconds (float)
with serial.Serial(sys.argv[1], timeout=2) as s:
    # print(cmd('AT+CBC')) # get battery level
    # print(cmd('AT+CSPN?')) # get operator
    # print(cmd('AT+CGMR')) # get firmware version

    # print(cmd('AT+CSQ')) # get signal strength

    # print(cmd('AT'))
    # print(cmd('AT+CUSD')) # get unstructured service data
    # print(cmd('AT+CMEE=2')) # set verbose error mode
    # print(cmd('AT+CPIN?')) # get SIM status
    # print(cmd('AT+CIPGSMLOC=1,1')) # get location 1

    # print(cmd('AT+CMGF=1')) # set text mode (this is required for AT+CMGL="ALL"
    # print(cmd('AT+CMGL="ALL"')) # get all SMS
    # print(cmd('AT+CMGD=1,4')) # delete all SMS
    # AT+CMGDA Delete All SMS

    # print(cmd('AT+CWHITELIST?')) # show whitelist
    # print(cmd('AT+CWHITELIST=1')) # enable call whitelist

    # cmd('AT+CMGDA') # Delete All SMS
    # cmd('AT+CWHITELIST=1') # enable call whitelist
    cmd('AT+CMEE=2') # set verbose error mode
    send_data(sys.argv[2], sys.argv[3], sys.argv[4])

