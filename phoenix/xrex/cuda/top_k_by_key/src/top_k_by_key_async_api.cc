
#include "nanobind/nanobind.h"
#include "top_k_by_key_async_kernel.hpp"

namespace nb = nanobind;

XLA_FFI_DEFINE_HANDLER_SYMBOL(
    top_k_by_key_async,
    top_k_by_key_bf16_async,
    ffi::Ffi::Bind()
        .Ctx<ffi::PlatformStream<cudaStream_t>>()
        .Ctx<ffi::ScratchAllocator>()
        .Arg<ffi::Buffer<ffi::DataType::BF16>>()
        .Attr<int64_t>("k")
        .Ret<ffi::Buffer<ffi::DataType::BF16>>()
        .Ret<ffi::Buffer<ffi::DataType::S32>>()
);

template <typename T>
nb::capsule encapsulate_ffi_call(T* fn) {
  static_assert(
      std::is_invocable_r_v<XLA_FFI_Error*, T, XLA_FFI_CallFrame*>,
      "Encapsulated function must be an XLA FFI handler"
  );
  return nb::capsule(reinterpret_cast<void*>(fn));
}

NB_MODULE(top_k_by_key_async_api, m) {
  m.doc() = "Top-k by key (BF16) - async h-indexer";
  m.def("top_k_by_key_async", []() { return encapsulate_ffi_call(top_k_by_key_async); });
}
