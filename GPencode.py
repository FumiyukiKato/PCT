import geohash


def encode(time, time_start, time_end, latitude, longtitude, theta_t=1440, theta_l=10):
    """
    encode by geohash encoding + periodical encoding

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
    digit = len(str(theta_t))
    t_hash = periodical_encode(time, time_start, time_end, theta_t, digit)
    g_hash = geohash_encode(latitude, longtitude, theta_l)
    return g_hash + t_hash


def periodical_encode(time, t_start, t_end, theta_t, digit):
    return str(theta_t*(time - t_start) // (t_end - t_start)).zfill(digit)
    
def geohash_encode(latitude, longtitude, zoom_level):
    return geohash.encode(latitude, longtitude, zoom_level)