#pragma once

#include <cstddef>
#include <stdexcept>

#include "xla/ffi/api/ffi.h"

namespace ffi = xla::ffi;

namespace xrex_cuda {

class ThrustAllocator {
 public:
  typedef char value_type;

  ThrustAllocator(ffi::ScratchAllocator* scratch_allocator)
      : scratch_allocator_(scratch_allocator) {}
  ~ThrustAllocator() = default;
  ThrustAllocator(const ThrustAllocator&) = delete;
  ThrustAllocator& operator=(const ThrustAllocator&) = delete;

  char* allocate(std::ptrdiff_t n) {
    auto result = scratch_allocator_->Allocate(n);
    if (!result.has_value()) {
      throw std::runtime_error("Failed to allocate memory");
    }
    return static_cast<char*>(result.value());
  }

  void deallocate(char* p, size_t n) {
  }

 private:
  ffi::ScratchAllocator* scratch_allocator_;
};

}
