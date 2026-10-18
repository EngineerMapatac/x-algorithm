#include <zlib.h>

#include "nanobind/nanobind.h"
#include "xla/ffi/api/c_api.h"
#include "xla/ffi/api/ffi.h"

namespace ffi = xla::ffi;
namespace nb = nanobind;

ffi::Error XaiAdler32CpuImpl(
    const ffi::AnyBuffer input,
    ffi::ResultBufferR0<ffi::U32> sum1,
    ffi::ResultBufferR0<ffi::U32> sum2
) {
  uint32_t init = adler32(0L, Z_NULL, 0);
  uint32_t checksum = adler32_z(init, (const Bytef*)input.untyped_data(), input.size_bytes());

  sum1->typed_data()[0] = checksum & 0xFFFF;
  sum2->typed_data()[0] = checksum >> 16;

  return ffi::Error::Success();
}

XLA_FFI_DEFINE_HANDLER_SYMBOL(
    XaiAdler32Cpu,
    XaiAdler32CpuImpl,
    ffi::Ffi::Bind()
        .Arg<ffi::AnyBuffer>()
        .Ret<ffi::BufferR0<ffi::U32>>()
        .Ret<ffi::BufferR0<ffi::U32>>(),
    {xla::ffi::Traits::kCmdBufferCompatible}
);

template <typename T>
nb::capsule encapsulate_ffi_call(T* fn) {
  return nb::capsule(reinterpret_cast<void*>(fn));
}

NB_MODULE(adler32_cpu_api, m) {
  m.doc() = "xrex.cuda adler32 cpu ops";
  m.def("adler32", []() { return encapsulate_ffi_call(XaiAdler32Cpu); });
}
