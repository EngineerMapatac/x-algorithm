
#include "nanobind/nanobind.h"
#include "unique_kernel.hpp"

namespace nb = nanobind;

XLA_FFI_DEFINE_HANDLER_SYMBOL(
    unique,
    unique_impl_int32,
    ffi::Ffi::Bind()
        .Ctx<ffi::PlatformStream<cudaStream_t>>()
        .Ctx<ffi::ScratchAllocator>()
        .Arg<ffi::BufferR1<ffi::DataType::S32>>()
        .Attr<bool>("return_inverse")
        .Attr<int64_t>("size")
        .Attr<int64_t>("fill_value")
        .Ret<ffi::BufferR1<ffi::DataType::S32>>()
        .Ret<ffi::BufferR1<ffi::DataType::S32>>()
        .Ret<ffi::BufferR1<ffi::DataType::S32>>()
        .Ret<ffi::BufferR1<ffi::DataType::S32>>()
        .Ret<ffi::BufferR1<ffi::DataType::S32>>()
);

template <typename T>
nb::capsule encapsulate_ffi_call(T* fn) {
  static_assert(
      std::is_invocable_r_v<XLA_FFI_Error*, T, XLA_FFI_CallFrame*>,
      "Encapsulated function must be an XLA FFI handler"
  );
  return nb::capsule(reinterpret_cast<void*>(fn));
}

NB_MODULE(unique_api, m) {
  m.doc() = "Unique kernel.";
  m.def("unique", []() { return encapsulate_ffi_call(unique); });
}
