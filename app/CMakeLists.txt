cmake_minimum_required(VERSION 3.24)

project(tech.flecs.fence)

if (NOT CMAKE_BUILD_TYPE)
    set(CMAKE_BUILD_TYPE Debug)
endif()

# frontend
add_custom_target(frontend ALL
    COMMAND npm ci
    COMMAND npm run build --if-present
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/frontend
    VERBATIM
)

# backend
if("${CMAKE_BUILD_TYPE}" STREQUAL "Debug")
    set(CARGO_PROFILE dev)
elseif("${CMAKE_BUILD_TYPE}" STREQUAL "RelWithDebInfo")
    set(CARGO_PROFILE release-with-debug)
else()
    set(CARGO_PROFILE release)
endif()

add_custom_target(backend ALL
    COMMAND cargo build ${CARGOFLAGS} --profile=${CARGO_PROFILE}
    WORKING_DIRECTORY ${CMAKE_SOURCE_DIR}/backend
    VERBATIM
)

add_custom_target(distclean
    COMMAND ${CMAKE_MAKE_PROGRAM} clean
    COMMAND rm -rf ${CMAKE_SOURCE_DIR}/backend/target/
    COMMAND rm -rf ${CMAKE_SOURCE_DIR}/frontend/node_modules/
    COMMAND rm -rf ${CMAKE_SOURCE_DIR}/frontend/dist/
    VERBATIM
)
