cmake_minimum_required(VERSION 3.10)

project(deadbeef-plugin-discord-rpc C)

set(PLUGIN_NAME discordrpc)

set(${PLUGIN_NAME}_SRC
    discordrpc.c
)

add_library(${PLUGIN_NAME} MODULE
    ${${PLUGIN_NAME}_SRC}
)

set_target_properties(${PLUGIN_NAME} PROPERTIES
    POSITION_INDEPENDENT_CODE ON
)

find_path(DISCORDRPC_INCLUDE_DIR
    NAMES discord_rpc.h rpc.h
    PATH_SUFFIXES discord-rpc
    PATHS /usr/include /usr/local/include
)

find_library(DISCORDRPC_LIBRARY
    NAMES discord-rpc discord_rpc libdiscord-rpc
    PATHS /usr/lib /usr/local/lib
)

find_path(DEADBEEF_INCLUDE_DIR
    NAMES deadbeef.h
    PATH_SUFFIXES deadbeef
)

if(NOT DISCORDRPC_INCLUDE_DIR)
    message(FATAL_ERROR "Could not find discord-rpc headers")
endif()

if(NOT DISCORDRPC_LIBRARY)
    message(FATAL_ERROR "Could not find libdiscord-rpc.so")
endif()

if(NOT DEADBEEF_INCLUDE_DIR)
    message(FATAL_ERROR "Deadbeef headers not found!")
endif()

message(STATUS "Found Deadbeef include path: ${DEADBEEF_INCLUDE_DIR}")

target_include_directories(${PLUGIN_NAME} PRIVATE
    ${DISCORDRPC_INCLUDE_DIR}
    ${DEADBEEF_INCLUDE_DIR}
)

target_link_libraries(${PLUGIN_NAME} PRIVATE
    ${DISCORDRPC_LIBRARY}
)

target_compile_features(${PLUGIN_NAME} PRIVATE c_std_99)
