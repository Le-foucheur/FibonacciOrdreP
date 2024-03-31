#include <CL/cl_platform.h>
#include <stdio.h>
#include <stdlib.h>
#ifdef __APPLE__
#include <OpenCL/opencl.h>
#include <unistd.h>
#else
#include <CL/cl.h>
#endif

#include "err_code.h"

typedef struct {
  cl_context ctx;
  cl_device_id id;
  cl_command_queue queue;
} full_executor;

cl_uint num_plateform;
cl_platform_id *platforms;
cl_uint *num_devices;
cl_device_id **all_devices;
cl_uint num_all_devices;
full_executor *all_context;

void notify_err ( const char* errinfo, const void* private_info, size_t cb, void* user_data){
  printf("Runtime OpenCL error: %s",errinfo);
}

cl_int setup(){

  //get available platforms
  cl_int err;
  err = clGetPlatformIDs(0,NULL,&num_plateform);
  if (err!=CL_SUCCESS) return err;
  platforms = malloc(sizeof(cl_platform_id)*num_plateform);
  if (platforms==NULL) return CL_OUT_OF_HOST_MEMORY;
  err = clGetPlatformIDs(num_plateform, platforms, NULL);
  if (err!=CL_SUCCESS) return err;
  
  all_devices = malloc(sizeof(cl_device_id*)*num_plateform);
  if (all_devices==NULL) return CL_OUT_OF_HOST_MEMORY;
  num_devices = malloc(sizeof(cl_uint)*num_plateform);
  if (num_devices==NULL) return CL_OUT_OF_HOST_MEMORY;

  
  //get devices on each platform, more or less error resilient
  for (cl_uint i=0; i<num_plateform; i++) {
    err = clGetDeviceIDs(platforms[i], CL_DEVICE_TYPE_ALL, 0, NULL, num_devices+i);
    all_devices[i] = malloc(sizeof(cl_device_id)*num_devices[i]);
    if (err==CL_SUCCESS) {
       err = clGetDeviceIDs(platforms[i], CL_DEVICE_TYPE_ALL, num_devices[i], all_devices[i], NULL);
    }
    if (err==CL_SUCCESS) {
      num_all_devices += num_devices[i];
    }
    else {
      num_devices[i] = 0;
      printf("Opencl Warning: %s",err_code(err));
    }
  }
  if (num_all_devices==0){printf("No OpenCl device");return CL_DEVICE_NOT_FOUND;}

  //create contexts and queues for each devices
  all_context = malloc(sizeof(full_executor)*num_all_devices);
  if (all_context==NULL) return CL_OUT_OF_HOST_MEMORY;
  cl_queue_properties queue_properties[3] = {CL_QUEUE_PROPERTIES,CL_QUEUE_OUT_OF_ORDER_EXEC_MODE_ENABLE,0};//|CL_QUEUE_ON_DEVICE|CL_QUEUE_ON_DEVICE_DEFAULT,0};
  cl_uint tmp=0;
  for (cl_uint i=0; i<num_plateform; i++) {
    cl_context_properties context_properties[3] = {CL_CONTEXT_PLATFORM,(cl_context_properties)(platforms[i]),0};
    for (cl_uint j=0; j<num_devices[i]; j++){
      //context
      all_context[tmp]=(full_executor){clCreateContext(context_properties,1,all_devices[i]+j,&notify_err,NULL,&err),all_devices[i][j],0};
      if (err==CL_SUCCESS){//queues
        all_context[tmp].queue=clCreateCommandQueueWithProperties(all_context[tmp].ctx,all_context[tmp].id,queue_properties,&err);
      }
      if (err!=CL_SUCCESS){//error handling
        printf("warning, device ignored due to following error: %s\n",err_code(err));
        num_all_devices-=1;
      }else tmp+=1;
    }
  }
  return CL_SUCCESS;
}

int main(){
  cl_int err;
  err = setup();
  if (err != CL_SUCCESS){
    printf("Opencl Error: %s",err_code(err));
    exit(EXIT_FAILURE);
  }
  if (num_all_devices==0){
    printf("ERROR:No usable device detected, exiting");
    exit(EXIT_FAILURE);
  } else {
    printf("Number of OpenCl device to perform calculation: %u",num_all_devices);
  }
  return 0;
}
