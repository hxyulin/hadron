# Drivers

The kernel drivers are responsible for interfacing with hardware devices.
After initialization, the driver should register itself with the corresponding subsystem in the kernel.

## Finding Drivers

### Built-in Drivers

For built-in kernel drivers, driver-type specific code is stored in the `.drivers` section of the kernel binary.
This should contain static information about the driver, as well as a way to match the driver to a specific device.
This should also contain the `probe` function for a driver, which is called when a driver is matached to a device.
The 'probe' function can be used to ensure that the driver is compatible with the device, and to initialize the driver.
NOTE: The design is not yet finalized, and the arguments and return values are not yet defined.

### Module-based Drivers

For module-based drivers, the driver is loaded as an elf module, and the driver static information is stored at runtime in a dynamic array.
NOTE: The module-based drivers are not yet implemented.

## Driver Types

There are currently two types of drivers

### Block Devices

Block devices are devices that allow reading and writing to a block of data.

### GPU Devices

GPU devices are devices that allow rendering to a framebuffer.

The GPU drivers are inspired by the linux driver model, and use the DRM / KMS API to communicate with the GPU.

#### DRM

The DRM (Direct Rendering Manager) API is a low-level API for rendering to a framebuffer.
A DRM driver consists of a number of connectors, encoders, crtcs, and planes.
Each connector is a physical connection to a GPU, and each encoder is a way to encode data to a connector.
Each crtc is a way to display a connector, and each plane is a way to render to a crtc.

#### KMS

The KMS (Kernel Mode Setting) API is a high-level API for configuring the framebuffer.
It is used to configure the resolution, depth, and other settings of the framebuffer.

# API

The driver API is currently not finalized, and is subject to change.
The current API are only rust functions, and are not yet documented.
