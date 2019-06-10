
from lib import *
import numpy as np

import weld.bindings as weld

import time

# Input data size
size = (1 << 26)
print("size: {} MB".format(size >> 20))


dtype="float64"
a_orig = np.array(np.random.rand((size)), dtype=dtype)

weld.weld_set_log_level(weld.WeldLogLevelDebug)

print "Running Weld..."
start = time.time()
a = HelloWeldVector(a_orig)
a = a.exp()
if dtype == "float32":
    res = a.weldobj.evaluate(WeldVec(WeldFloat()), verbose=True)
elif dtype == "float64":
    res = a.weldobj.evaluate(WeldVec(WeldDouble()), verbose=True)
else:
    raise Exception()
end = time.time()
print "Weld result:", res
weld_time = (end - start)
print "({:.3} seconds)".format(weld_time)

print "Running NumPy..."
start = time.time()
res = np.exp(a_orig)
end = time.time()
np_time = (end - start)
print "({:.3} seconds)".format(np_time)

print "NumPy result:", res
