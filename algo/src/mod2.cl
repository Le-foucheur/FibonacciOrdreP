#include <opencl-c-base.h>
//This whole implem ASSUME LITTLE ENDIAN, do NOT use it on BIG ENDIAN computers/accélérator. (this should be the case in most situations anyway ...)

char arr_getb(__global char* src,ulong index){
	char tmp=src[index/8];
	return (tmp>>index%8)&1;
}
char16 arr_get16c(__global char* src,ulong index){
	return vload16(0,src+index);
}

__kernel void jump_formulae_internal(
		__global char* src,
		__global char* dst,
		const ulong p,
		const uchar less_one //actually, bool 0 or 1
		){
	
    char16 acc1=0,acc2=0,acc3=0,acc4=0,acc5=0,acc7=0,acc8=0;

	ulong k = get_global_id(0)*15;
	if (arr_getb(src, less_one)) {
    	
    }
	
}
