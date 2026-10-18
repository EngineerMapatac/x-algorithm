
#include "adler32_api.h"

#include "nanobind/nanobind.h"
#include "xla/ffi/api/ffi.h"

namespace nb = nanobind;
namespace ffi = xla::ffi;

XLA_FFI_DEFINE_HANDLER_SYMBOL(
    XaiAdler32,
    XaiAdler32Host,
    ffi::Ffi::Bind()
        .Ctx<ffi::PlatformStream<cudaStream_t>>()
        .Arg<ffi::AnyBuffer>()
        .Ret<ffi::BufferR0<ffi::U32>>()
        .Ret<ffi::BufferR0<ffi::U32>>(),
    {xla::ffi::Traits::kCmdBufferCompatible}
);

XLA_FFI_DEFINE_HANDLER_SYMBOL(
    XaiGlobalAdler32Shard,
    XaiGlobalAdler32ShardHost,
    ffi::Ffi::Bind()
        .Ctx<ffi::PlatformStream<cudaStream_t>>()
        .Arg<ffi::AnyBuffer>()
        .Arg<ffi::Buffer<ffi::U32>>()
        .Arg<ffi::Buffer<ffi::U32>>()
        .Arg<ffi::Buffer<ffi::U32>>()
        .Ret<ffi::BufferR1<ffi::U32>>(),
    {xla::ffi::Traits::kCmdBufferCompatible}
);

XLA_FFI_DEFINE_HANDLER_SYMBOL(
    XaiGlobalAdler32ShardFast,
    XaiGlobalAdler32ShardFastHost,
    ffi::Ffi::Bind()
        .Ctx<ffi::PlatformStream<cudaStream_t>>()
        .Arg<ffi::AnyBuffer>()
        .Arg<ffi::Buffer<ffi::U32>>()
        .Attr<int>("divider_log2")
        .Ret<ffi::BufferR1<ffi::U32>>(),
    {xla::ffi::Traits::kCmdBufferCompatible}
);

template <typename T>
nb::capsule encapsulate_ffi_call(T* fn) {
  static_assert(
      std::is_invocable_r_v<XLA_FFI_Error*, T, XLA_FFI_CallFrame*>,
      "Encapsulated function must be and XLA FFI handler"
  );
  return nb::capsule(reinterpret_cast<void*>(fn));
}

NB_MODULE(adler32_api, m) {
  m.doc() = "Adler32";
  m.def("adler32", []() { return encapsulate_ffi_call(XaiAdler32); });
  m.def("global_adler32_shard", []() { return encapsulate_ffi_call(XaiGlobalAdler32Shard); });
  m.def("global_adler32_shard_fast", []() {
    return encapsulate_ffi_call(XaiGlobalAdler32ShardFast);
  });
}
