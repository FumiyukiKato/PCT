import math

# const for quadkey encoding
MAX_LONGITUDE = 180.0
MAX_LATITUDE = 85.05112877980659 # (2*math.atan2(exp(math.pi))*180.0/math.pi - 90.0) 
MAX_ZOOM = 31
MAX_THETA_T = 32
MIN_LONGITUDE = - MAX_LONGITUDE
MIN_LATITUDE = - MAX_LATITUDE
BASE8 = {
    '000': '0',
    '001': '1',
    '010': '2',
    '011': '3',
    '100': '4',
    '101': '5',
    '110': '6',
    '111': '7',
}

def encode(time, time_start, time_end, latitude, longtitude, theta_t=23, theta_l=16, N=8):
    """
    TrajectoryHash based encoding

    Parameters
    ----------
    time : int
        UNIXEPOCH
            ex 1605398680
    latitude : float
        coordinate
            ex 35.5555
    longtitude: float
        coordinate
            ex 50.0001
    theta_t: int
        granularity parameter for time, period is distributed by theta_t small periods
            ex 1000
    theta_l: int
        granularity parameter for location, corresponding geohash zoom_level
            ex 10
  
    Returns
    -------
    hash: str
        hash
    """
    assert theta_l < MAX_ZOOM,  "theta_l has to be  less than %d " %  MAX_ZOOM
    assert theta_t <= MAX_THETA_T, "theta_t has to be  less than %d " %  MAX_THETA_T
    
    b1, b2 = quadkeyEncoding(longtitude, latitude, theta_l)
    maxlength = getMaxBinaryLength(time_start, time_end)
    b3 = periodicalEncoding(time, time_start, theta_t, maxlength)
    b1, b2, b3 = maxPadding(b1, b2, b3)
    
    binary = mix(b1, b2, b3)

    th = base8Encoding(binary)
    
    return th


def mix(b1, b2, b3):
    binary = []
    for bit1, bit2, bit3 in zip(b1, b2, b3):
        binary.append(bit1)
        binary.append(bit2)
        binary.append(bit3)
    return binary


def base8Encoding(binary):    
    return ''.join([ BASE8.get("".join(binary[i:i+3])) for i in range(0, len(binary), 3)])

def periodicalEncoding(time, time_start, theta_t, maxlength):
    t_diff = time - time_start
    shift = 32 - theta_t
    t_diff = t_diff >> shift
    binary = bin(t_diff)[2:2+maxlength]
    return binary
    
    
def maxPadding(b1, b2, b3):
    assert len(b1) == len(b2), "the length of b1 must be equals to b2"
    
    len1 = len(b1)
    len3 = len(b3)
    if len1 > len3:
        b3 = zeroPadding(b3, len1)
    else:
        b1 = zeroPadding(b1, len3)
        b2 = zeroPadding(b2, len3)

    return b1, b2, b3
    
    
def zeroPadding(binaryStr, maxlength):
    lengthOfbinary = len(binaryStr)
    if lengthOfbinary >= maxlength:
        return binaryStr[(lengthOfbinary - maxlength):]
    else:
        return ''.join(['0']*(maxlength - lengthOfbinary)) + binaryStr
    

def getMaxBinaryLength(t_start, t_end):
    t_diff = t_end - t_start
    lenOfDiff = len(bin(t_diff))-2
    if lenOfDiff <= 8:
        return 8
    elif lenOfDiff <= 16:
        return 16
    elif lenOfDiff <= 32:
        return 32
    elif lenOfDiff <= 64:
        return 64
    else:
        raise ValueError("Invalid size of time data.")


def quadkeyEncoding(lon, lat, zoom):
    lon = min(MAX_LONGITUDE, max(MIN_LONGITUDE, lon))
    lat = min(MAX_LATITUDE, max(MIN_LATITUDE, lat))
    
    # TransformToPixelCoodinate
    fx = (lon+180.0)/360.0;
    sinlat = math.sin(lat * math.pi/180.0);
    fy = 0.5 - math.log((1+sinlat)/(1-sinlat)) / (4*math.pi);
    
    # 2**zoom
    mapsize = 1 << zoom
    
    x = math.floor(fx*mapsize)
    y = math.floor(fy*mapsize)
    x = min(mapsize - 1, max(0, x))
    y = min(mapsize - 1, max(0, y))
   
    return zeroPadding(bin(x)[2:], zoom), zeroPadding(bin(y)[2:], zoom)
