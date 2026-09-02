"""Minimal QuickDraw PICT v1 decoder for the 1-bit StuntCopter resources."""
import struct
import numpy as np


def read_ad_rsrc(path):
    d = open(path, 'rb').read()
    n = struct.unpack('>H', d[24:26])[0]
    for i in range(n):
        eid, eoff, elen = struct.unpack('>III', d[26 + i * 12:26 + i * 12 + 12])
        if eid == 2:
            return d[eoff:eoff + elen]


def get_pict(rf, want_id):
    dataOff, mapOff, dataLen, mapLen = struct.unpack('>IIII', rf[:16])
    m = rf[mapOff:mapOff + mapLen]
    tlo, nlo = struct.unpack('>HH', m[24:28])
    tl = m[tlo:]
    nt = struct.unpack('>H', tl[:2])[0] + 1
    for i in range(nt):
        e = tl[2 + i * 8:2 + i * 8 + 8]
        if e[:4] == b'PICT':
            cnt = struct.unpack('>H', e[4:6])[0] + 1
            rlo = struct.unpack('>H', e[6:8])[0]
            for j in range(cnt):
                r = m[tlo + rlo + j * 12: tlo + rlo + j * 12 + 12]
                rid = struct.unpack('>h', r[:2])[0]
                dO = struct.unpack('>I', b'\x00' + r[5:8])[0]
                if rid == want_id:
                    L = struct.unpack('>I', rf[dataOff + dO:dataOff + dO + 4])[0]
                    return rf[dataOff + dO + 4:dataOff + dO + 4 + L]


def unpackbits(buf, off, want):
    out = bytearray()
    i = off
    while len(out) < want:
        n = buf[i]; i += 1
        if n < 128:
            out += buf[i:i + n + 1]; i += n + 1
        elif n > 128:
            out += bytes([buf[i]]) * (257 - n); i += 1
    return bytes(out[:want]), i


def decode_pict(p):
    """Return an ink boolean array for the (single) bitmap in a v1 1-bit PICT."""
    size = struct.unpack('>H', p[:2])[0]
    top, left, bottom, right = struct.unpack('>hhhh', p[2:10])
    i = 10
    result = None
    while i < len(p):
        op = p[i]; i += 1
        if op == 0x00:            # nop
            continue
        if op == 0x11:            # version
            i += 1
        elif op == 0x01:          # clipRgn
            rsize = struct.unpack('>H', p[i:i + 2])[0]
            i += rsize
        elif op in (0x90, 0x98):  # BitsRect / PackBitsRect (BitMap form)
            rowBytes = struct.unpack('>H', p[i:i + 2])[0]; i += 2
            bt, bl, bb, br = struct.unpack('>hhhh', p[i:i + 8]); i += 8
            i += 8 + 8 + 2        # srcRect, dstRect, mode
            h, w = bb - bt, br - bl
            rows = []
            for _ in range(h):
                if rowBytes < 8:
                    row = p[i:i + rowBytes]; i += rowBytes
                else:
                    if rowBytes > 250:
                        cnt = struct.unpack('>H', p[i:i + 2])[0]; i += 2
                    else:
                        cnt = p[i]; i += 1
                    row, _ = unpackbits(p, i, rowBytes); i += cnt
                rows.append(row)
            bits = np.zeros((h, rowBytes * 8), dtype=bool)
            for y, row in enumerate(rows):
                for x in range(len(row)):
                    for b in range(8):
                        bits[y, x * 8 + b] = (row[x] >> (7 - b)) & 1
            result = bits[:, :w]   # 1 = black in QuickDraw
        elif op == 0xA0:          # short comment
            i += 2
        elif op == 0xA1:          # long comment
            n = struct.unpack('>H', p[i + 2:i + 4])[0]; i += 4 + n
        elif op == 0xFF:          # end
            break
        else:
            # Unknown opcode -- report and stop so we can extend the decoder.
            print(f'  ! unhandled opcode 0x{op:02x} at {i-1}')
            break
    return result, (right - left, bottom - top)


if __name__ == '__main__':
    import sys
    from PIL import Image
    rf = read_ad_rsrc(r'D:\training\opensc\reference\stuntcopter\StuntCopter.Rsrc.appledouble')
    names = {130: 'scorebox', 128: 'copter', 129: 'man', 356: 'cloud1', 357: 'cloud2', 358: 'cloud3'}
    for pid, name in names.items():
        p = get_pict(rf, pid)
        bits, (w, h) = decode_pict(p)
        if bits is None:
            print(f'PICT {pid} ({name}): no bitmap decoded')
            continue
        img = np.full((bits.shape[0], bits.shape[1], 3), 255, np.uint8)
        img[bits] = 0
        out = rf'C:\Users\MikaelBeyene\AppData\Local\Temp\claude\D--training-opensc\68a8ed9d-2ec2-4fb7-9ee2-ba32e48ffa1f\scratchpad\pict_{name}.png'
        Image.fromarray(img).save(out)
        print(f'PICT {pid} ({name}): frame {w}x{h}, bitmap {bits.shape[1]}x{bits.shape[0]} -> {name}.png')
