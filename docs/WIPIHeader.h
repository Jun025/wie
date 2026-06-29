
/* 
   다음의 string.h 의 다음의 함수가 사용가능합니다.
   strcpy, strncpy, strcat, strncat, strcmp, strncmp, strchr, strrchr, strspn, 
   strcspn, strpbrk, strstr, strlen, strtok, memcpy, memmove, memcmp, memchr, 
   memset
 */
#include "string.h"

/* 
   다음의 stdlib.h 의 다음의 함수가 사용가능합니다.
   atof, atoi, atoll, strtod, strtol, strtoul
 */
#include "stdlib.h"

/*
   다음의 stdarg.h 의 다음의 함수가 사용가능합니다.
   va_list, va_start, va_arg, va_end
 */
#include "stdarg.h"

/*
   다음의 stdarg.h 의 다음의 함수가 사용가능합니다.
   clock, time, difftime, mktime, localtime, gmtime
 */
#include "time.h"

/* C API에서 사용하는 모든 error코드를 정의 합니다.  */
#include "Merror.h"
//#include "MHal.h"

struct _JString;
typedef  unsigned char      M_Boolean;     
typedef  unsigned int  M_Uint32;
typedef  unsigned short     M_Uint16;
typedef  unsigned char      M_Uint8;
typedef char  M_Char;
typedef  signed int    M_Int32;
typedef  signed short       M_Int16;
typedef  signed char        M_Int8;
typedef  unsigned char     M_Byte; 
typedef  signed __int64   M_Int64; 
typedef  unsigned __int64 M_Uint64; 
typedef unsigned short      M_UCode;
typedef  signed long       M_Sint31;   
typedef  signed short      M_Sint15;
typedef M_Uint32  Addr;
typedef M_Uint64  ulong64;
typedef signed __int64   long64;

typedef unsigned __int64 uint64;
typedef __int64    int64;
typedef unsigned char   uint8;
typedef unsigned short  uint16;
typedef unsigned int    uint32;
typedef signed char     int8;
typedef signed short    int16;
typedef signed int      int32;
typedef float           float32;
typedef double          float64;
typedef uint8	byte;
typedef uint16	word;
typedef	uint16		jchar;
typedef	void*		jref;
typedef	jref		jobject;
typedef	uint16		jchar;
typedef	int8		jbyte;
typedef	int16 		jshort;
typedef	int32		jint;
typedef int64		jlong;
typedef	jint		jsize;
typedef	float		jfloat;
typedef	double		jdouble;
typedef 	jobject		jarray;
typedef	uint32		jboolean;
typedef 	char 		Name;

typedef struct _OBJECT_HEAD {
    M_Int32   objInfo;
} OBJECT_HEAD;

typedef struct _ARROBJECT_HEAD {
    M_Int32 objInfo;
    M_Uint32 length;
} ARROBJECT_HEAD;

typedef struct _IndirectBase {
        void* base;
        struct _IndirectBase* next;
} IndirectBase;

#define OBJECT_HEADSIZE         sizeof(OBJECT_HEAD)
#define ARRAYOBJECT_HEADSIZE    sizeof(ARROBJECT_HEAD)
#define D_objBase(obj)      (((IndirectBase*)(obj))->base)

typedef	struct _JString* 	jstring;
typedef struct _JString {

	OBJECT_HEAD head;

	uint16* charArray;
	int  offset;
	int  count;

} JString;

#define getArrayLength(array)                   ((ARROBJECT_HEAD*)(D_objBase(array)))->length
#define D_byteArrayElement(array, index)        *((jbyte*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_charArrayElement(array, index)        *((jchar*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_shortArrayElement(array, index)        *((jshort*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_intArrayElement(array, index)        *((jint*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_floatArrayElement(array, index)        *((jfloat*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_longArrayElement(array, index)        *((jlong*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_doubleArrayElement(array, index)        *((jdouble*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_objectArrayElement(array, index)      *((Addr*)((uint32)(D_objBase(array))+ARRAYOBJECT_HEADSIZE) + index)
#define D_object(obj, off)      *((jint*)((char*)(D_objBase(obj))+OBJECT_HEADSIZE+off))
#define D_objectFloat(obj, off)      *((jfloat*)((char*)(D_objBase(obj))+OBJECT_HEADSIZE+off))
#define D_objectLong(obj, off)  *((jlong*)((char*)(D_objBase(obj))+OBJECT_HEADSIZE+off))
#define D_objectDouble(obj, off)  *((jdouble*)((char*)(D_objBase(obj))+OBJECT_HEADSIZE+off))

#define STRING_SIZE(STR) (((jstring)STR)->count)
#define STRING_DATA(STR) ((jchar*)&D_charArrayElement(((jstring)STR)->charArray, ((jstring)STR)->offset))

#ifndef NULL
#define NULL		0
#endif

#ifndef TRUE
#define TRUE	1
#endif

#ifndef FALSE
#define FALSE	0
#endif

#define inline	


/**
 * Key Code 값. 
 */
typedef enum MH_KeyCode{
	///
	MH_KEY_0		= '0',
	///
	MH_KEY_1		= '1',
	///
	MH_KEY_2		= '2',
	///
	MH_KEY_3		= '3',
	///
	MH_KEY_4		= '4',
	///
	MH_KEY_5		= '5',
	///
	MH_KEY_6		= '6',
	///
	MH_KEY_7 		= '7',
	///
	MH_KEY_8		= '8',
	///
	MH_KEY_9		= '9',
	///
	MH_KEY_ASTERISK	= '*',
	///
	MH_KEY_POUND		= '#',
	///
	MH_KEY_UP		= -1,
	///
	MH_KEY_DOWN		= -2,
	///
	MH_KEY_LEFT		= -3,
	///
	MH_KEY_RIGHT		= -4,
	///
	MH_KEY_SELECT		= -5,
	///
	MH_KEY_SOFT1		= -6,
	///
	MH_KEY_SOFT2		= -7,
	///
	MH_KEY_SOFT3		= -8,
	///
	MH_KEY_SEND		= -10,
	///
	MH_KEY_END		= -11,
	///
	MH_KEY_POWER		= -12,
	///
	MH_KEY_SIDE_UP		= -13,
	///
	MH_KEY_SIDE_DOWN	= -14,
	///
	MH_KEY_SIDE_SEL	= -15,
	///
	MH_KEY_CLEAR		= -16,
	///
	MH_KEY_FLIPDOWN	= -17,
	///
	MH_KEY_FLIPUP		= -18,
	///
	MH_KEY_INVALID	= 0
} MH_KeyCode;

typedef enum MH_Event {
	/// 시스템을를 종료한다. 종료시켜주는 이벤트.
	MH_EXIT_EVENT = 1,
	/// 키가  눌릴때 알려 주는 이벤트.
	MH_KEY_PRESSEVENT,
	/// 키가 떼어질때 알려 주는 이벤트.
	MH_KEY_RELEASEEVENT,
	/// 키를 누르고 있는경우 알려 주는 이벤트.	
	MH_KEY_REPEATEVENT,	
	/// 타이머가 만료될때 알려 주는 이벤트.
	MH_TIMER_EVENT,
	/// SMS 메시지가 수신되었음을 알려 주는 이벤트.
	MH_SMS_EVENT,
	/// 전화가 왔음을 알려 주는 이벤트.
	MH_CALL_EVENT,
	///어넌시에이터(Annuciator) 정보가 변경 되었을 때 알려 주는 이벤트.
	MH_ANN_EVENT,
	/// 네트워크 관련 정보가 변경되었을 때 알려주는 이벤트. 
	MH_NETWORK_EVENT,
	/// 시리얼 통신  관련 정보가 변경되었을 때 알려주는 이벤트. 
	MH_SERIAL_EVENT,
	///사운드  관련 정보가 변경되었을 때 알려주는 이벤트. 
	MH_SOUND_EVENT
} MH_Event;


typedef enum MC_KeyCode{
	///
	MC_KEY_0		= '0',
	///
	MC_KEY_1		= '1',
	///
	MC_KEY_2		= '2',
	///
	MC_KEY_3		= '3',
	///
	MC_KEY_4		= '4',
	///
	MC_KEY_5		= '5',
	///
	MC_KEY_6		= '6',
	///
	MC_KEY_7 		= '7',
	///
	MC_KEY_8		= '8',
	///
	MC_KEY_9		= '9',
	///
	MC_KEY_ASTERISK	= '*',
	///
	MC_KEY_POUND		= '#',
	///
	MC_KEY_UP		= -1,
	///
	MC_KEY_DOWN		= -2,
	///
	MC_KEY_LEFT		= -3,
	///
	MC_KEY_RIGHT		= -4,
	///
	MC_KEY_SELECT		= -5,
	///
	MC_KEY_SOFT1		= -6,
	///
	MC_KEY_SOFT2		= -7,
	///
	MC_KEY_SOFT3		= -8,
	///
	MC_KEY_SEND		= -10,
	///
	MC_KEY_END		= -11,
	///
	MC_KEY_POWER		= -12,
	///
	MC_KEY_SIDE_UP		= -13,
	///
	MC_KEY_SIDE_DOWN	= -14,
	///
	MC_KEY_SIDE_SEL	= -15,
	///
	MC_KEY_CLEAR		= -16,
	///
	MC_KEY_FLIPDOWN	= -17,
	///
	MC_KEY_FLIPUP		= -18,
	///
	MC_KEY_INVALID	= 0
} MC_KeyCode;

#define MV_KEY_PRESS_EVENT          MH_KEY_PRESSEVENT
#define MV_KEY_RELEASE_EVENT        MH_KEY_RELEASEEVENT
#define MV_KEY_REPEAT_EVENT         MH_KEY_REPEATEVENT
#define MV_SMS_EVENT                MH_SMS_EVENT
#define MV_ANN_EVENT                MH_ANN_EVENT
#define MV_CALL_EVENT               MH_CALL_EVENT
#define MV_SMS_EVENT                MH_SMS_EVENT
/* 끝 */

/* 프로그램 관리 event */
#define MV_APP_EVENT                100
#define MV_CHILDSTART_EVENT                     101
#define MV_CHILDSTOP_EVENT                      102

/* application life cycle에 따른 이벤트( MV_APP_EVNENT의 sub event) */
#define MV_APP_STOP                 1 // obsolete for inappropriate name
#define MV_APP_PAUSE                1
#define MV_APP_RESUME               2
#define MV_APP_DESTORY              3 // obsolete for inappropriate name
#define MV_APP_DESTROY                          3
#define MV_APP_ACTIVE               4

/* 내부 시스템 이벤트는 0x1000대를 사용합니다.*/

#define MV_TIMER_EVENT                          0x1001
//#define MV_PPPOPEN_EVENT                        0x1002
//#define MV_SOCKET_EVENT                         0x1003
#define MV_NETWORK_EVENT                         0x1002
#define MV_SERIAL_EVENT                         0x1003

/* 사용자 이벤트는 0x5000대를 사용합니다. */
#define MV_USER_EVENT                           0x5000


#define MC_DIR_SYS_READ_REQ_MASK  0x01
#define MC_DIR_SYS_WRITE_REQ_MASK  0x02
#define MC_DIR_SHARED_READ_REQ_MASK  0x04
#define MC_DIR_SHARED_WRITE_REQ_MASK  0x08
#define MC_NETWORK_ACCESS_REQ_MASK  0x10
#define MC_SERIAL_ACCESS_REQ_MASK  0x20
#define MC_SYSTEM1_ACCESS_REQ_MASK  0x40
#define MC_SYSTEM2_ACCESS_REQ_MASK  0x80

#define MC_GETDPTR(indirectPtr)	((void*)((char*)(D_objBase(indirectPtr))+ARRAYOBJECT_HEADSIZE))

#pragma pack(1)
typedef struct _MTimer {
        int type;                       
        struct _MTimer* alarmNext;
        void* alarmWakupFunc;
        uint64 alarmTime;
} MTimer;
#pragma pack(0)

typedef struct _MCTimer {
        MTimer timer;
        void* parm;
        void* prg;
        void* cb;                      
} MCTimer;

typedef void (*TIMERCB)(MCTimer *tm, void* parm);

typedef M_Int32 (*MC_GrpPixelOpProc)(M_Int32 srcpxl, 
        M_Int32 orgpxl, M_Int32 param1);

struct _MC_GrpContext{
	int m_mask;
	// clipping region. if null clipping is disable.
	// clip[0] startx clip[1] starty clip[2] endx clip[3] endy
	int m_clip[4];
	// foreground pixel. if -1 is skip foreground.
	int m_fgpxl;
	// background pixel. if -1 is skip foreground.
	int m_bgpxl;
	// image transparent pixel.
	int m_transpxl;
	// alpha channel value.
	int m_alpha;
	// offsetX
	int m_offsetX;
	// offsetY
	int m_offsetY;
	// pixel function.
	MC_GrpPixelOpProc m_pfnPixelOp;
	// parameter 1.
	int m_param1;
	// parameter 2.
	int m_reserved;
	// font 
	int m_font;
	// style
	int m_style;
};

typedef struct _MC_GrpContext MC_GrpContext;
/**
 * 팔레트를 사용하지 않는 경우의 컬러 타입
 */
#define MC_GRP_DIRECT_COLOR_TYPE     (1 << 0)
/**
 * 흑백 타입
 */
#define MC_GRP_GRAY_TYPE       (1 << 1)
/**
 * 컬러 타입
 */
#define MC_GRP_COLOR_TYPE       (1 << 2)

/**
 * 화면 정보 구조체
 */
struct _MC_GrpDisplayInfo{
    /// 픽셀당 비트수
    int m_bpp;
    /// 실제적인 픽셀당 사용 비트수
    int m_depth;
    /// 화면의 픽셀 단위 폭
    int m_width;
    /// 화면의 픽셀 단위 높이
    int m_height;
    /// 프레임 버퍼의 화면의 한 줄당 바이트 수
    int m_bpl;
    /// 컬러 타입; MC_GRP_DIRECT_COLOR_TYPE, MC_GRP_GRAY_TYPE, MC_GRP_COLOR_TYPE
    int m_colortype;
    /// 빨간 색상 매스크
    int m_redmask;
    /// 파랑 색상 매스크
    int m_bluemask;
    /// 녹색 색상 매스크
    int m_greenmask;
};

/**
 * 화면의 각종 정보를 가집니다.
 */
typedef struct _MC_GrpDisplayInfo MC_GrpDisplayInfo;

struct _MC_GrpFrameBuffer{
    // 프레임 버퍼의 폭(Pixel 단위)
    M_Int32 w;
    // 프레임 버퍼의 높이(Pixel 단위)
    M_Int32 h;
    // 프레임 버퍼의 한 줄당 바이트 수; 패딩(Padding)되는 바이트까지 포함.
    M_Int32 bpl;
    // 프레임 버퍼의 한 픽셀당 비트 수
    M_Int32 bpp;
    // 실제적은 프레임 버퍼의 ID
    M_Int32 frameBufID;
};

/**
 * 프레임 버퍼.
 *
 * 내부에 높이와 넓이와 프레임 버퍼 포인터를 가집니다.
 */
typedef M_Int32 MC_GrpFrameBuffer;

/**
 * 프레임 버퍼의 포인터를 돌려줍니다.
 @param a [in] MC_GrpFrameBuffer
 @return 프레임 버퍼의 내용이 있는 포인터
 */
#define MC_GRP_GET_FRAME_BUFFER_POINTER(a) ((M_Int32*)MC_GETDPTR(((struct _MC_GrpFrameBuffer*)MC_GETDPTR(a))->frameBufID))


/**
 * 프레임 버퍼의 폭을 돌려줍니다.

 @param a [in] MC_GrpFrameBuffer
 @return 프레임 버퍼 포인터
 */
#define MC_GRP_GET_FRAME_BUFFER_WIDTH(a) ((struct _MC_GrpFrameBuffer *)MC_GETDPTR(a))->w

/**
 * 프레임 버퍼의 높이을 돌려줍니다.

 @param a [in] MC_GrpFrameBuffer
 @return 프레임 버퍼 높이
 */
#define MC_GRP_GET_FRAME_BUFFER_HEIGHT(a) ((struct _MC_GrpFrameBuffer *)MC_GETDPTR(a))->h

/**
 * 프레임 버퍼의 한줄당 바이트수를 돌려줍니다.

 @param a [in] MC_GrpFrameBuffer
 @return 프레임 버퍼 한줄당 바이트수
 */
#define MC_GRP_GET_FRAME_BUFFER_BPL(a) (((struct _MC_GrpFrameBuffer *)MC_GETDPTR(a)))->bpl

/**
 * 프레임의 한 픽셀당 비트수를 돌려줍니다.

 @param a [in] MC_GrpFrameBuffer
 @return 한 픽셀당 비트수
 */
#define MC_GRP_GET_FRAME_BUFFER_BPP(a) ((struct _MC_GrpFrameBuffer *)MC_GETDPTR(a))->bpp

/**
 * 클리핑 영역을 가르키는 사각형을 지정합니다.
 * 
 * 사각형은 왼쪽 상단의 점과 오른쪽 하단의 점으로 기술되며,
 * 왼쪽 상단의 점은 사각형에 포함되지만, 오른쪽 하단의 점은 사각형에
 * 포함되지 않습니다.
 */
#define MC_GRP_CONTEXT_CLIP_IDX             0

/**
 * 전경색 픽셀 값을 지정합니다.
 */
#define MC_GRP_CONTEXT_FG_PIXEL_IDX         1

/**
 * 후경색 픽셀 값을 지정합니다.
 */
#define MC_GRP_CONTEXT_BG_PIXEL_IDX         2

#define MC_GRP_CONTEXT_TRANS_PIXEL_IDX      3

/**
 * 그리기의 투명 정도를 지정합니다.
 *
 * 0이면 화면에 나오지 않고, 255이면 화면에 투명하지 않게 출력됩니다.
 */
#define MC_GRP_CONTEXT_ALPHA_IDX            4   

/**
 * 픽셀 연산(Operation)함수를 지정합니다.
 *
 * @see MC_GrpPixelOpFunc
 */
#define MC_GRP_CONTEXT_PIXELOP_IDX          5   

/**
 * 픽셀 연산 함수의 매개 변수를 지정합니다.
 *
 * 픽셀 연산 함수가 불릴때 넘어가는 세번째 파리미터를 지정합니다.
 */
#define MC_GRP_CONTEXT_PIXEL_PARAM1_IDX     6   

/**
 * 폰트 식별자를 지정합니다.
 *
 * MC_getFont함수를 통해서 얻어오는 폰트 식별자를 지정합니다.
 */
#define MC_GRP_CONTEXT_FONT_IDX             7   

/**
 * 선그리기 스타일을 지정합니다.
 *
 * MC_GRP_SOLID_STYLE 혹은 MC_GRP_DOTTED_STYLE둘 중에 하나가 
 * 됩니다.
 */
#define MC_GRP_CONTEXT_STYLE_IDX            8

/**
 * 그리기 모드를 지정합니다.
 *
 * 그리기시 XOR로 그릴지 여부를 정의합니다. 1이면 XOR로 그리고,
 * 그렇지 않으면 일반적인 모드로 그립니다.
 */
#define MC_GRP_CONTEXT_XOR_MODE_IDX         9

/**
 * 그리기 상대 좌표의 원점(Offset)을 지정합니다.
 *
 * 상대 좌표의 점 좌표를 정수 어레이에 넣어서 지정합니다.
 */
#define MC_GRP_CONTEXT_OFFSET_IDX			10

/**
 * 선 그리기시에 동일한 색상으로 그립니다.
 */
#define MG_FB_SOLID_STYLE 		    0
#define MC_GRP_SOLID_STYLE                  MG_FB_SOLID_STYLE

/**
 * 선 그리기시에 한 점그린후 다음점은 그리지 않는 식으로 반복해서 
 * 그립니다.
 */
#define MG_FB_DOTTED_STYLE		    1
#define MC_GRP_DOTTED_STYLE                 MG_FB_DOTTED_STYLE


#define MC_GRP_CONTEXT_CLIP_MASK			(1)
#define MC_GRP_CONTEXT_OFFSET_MASK			(1 << 1)


/**
 * 작은 폰트 크기를 지정합니다.
 */
#define MC_GRP_FT_SIZE_SMALL        8

/**
 * 중간 폰트 크기를 지정합니다.
 */
#define MC_GRP_FT_SIZE_MEDIUM       0

/**
 * 큰 폰트 크기를 지정합니다.
 */
#define MC_GRP_FT_SIZE_LARGE        16


/**
 * 시스템에서 사용하는 폰트 페이스를 지정합니다.
 */
#define MC_GRP_FT_FACE_SYSTEM       0

/**
 * 각 폰트의 폭이 균일한 폰트 페이스를 지정합니다.
 */
#define MC_GRP_FT_FACE_MONOSPACE    32

/**
 * 각 폰트의 폭이 균일하지 않은 폰트 페이스를 지정합니다.
 */
#define MC_GRP_FT_FACE_PROPORTIONAL 64

/**
 * 일반적은 스타일의 폰트를 지정합니다.
 */
#define MC_GRP_FT_STYLE_PLAIN       0

/**
 * 굵은 스타일의 폰트를 지정합니다.
 */
#define MC_GRP_FT_STYLE_BOLD        1

/**
 * 기울여진 스타일의 폰트를 지정합니다.
 */
#define MC_GRP_FT_STYLE_ITALIC      2

/**
 * 밑줄이 쳐진 스타일의 폰트를 지정합니다.
 */
#define MC_GRP_FT_STYLE_UNDERLINE   4


/**
 * 전체 이미지 소스 디코딩이 끝났음을 알립니다. 1로 정의되어 있습니다.
 */
#define MC_GRP_IMAGE_DONE        1

/**
 * 이미지 소스에서 부터 한 프레임이 이미지가 완성되었음을 알립니다. 0으로 
 * 정의되어 있습니다.
 */
#define MC_GRP_FRAME_DONE				0

/*
 * 이미지가 디코딩되어서 저장되는 구조체입니다.
 */
typedef struct MC_GrpImgTarget{
	// 이미지 프레임 버퍼가 됩니다. 
	MC_GrpFrameBuffer m_img;
    // 마스크 이미지의 프레임 버퍼가 됩니다.
	MC_GrpFrameBuffer m_mask;
    // 에니메이션일 경우 전체 에니메이션을 몇번 수행할지를 결정합니다.
	int m_loopcount;
    // 에니메이션 사이에 지연되는 값입니다. 밀리세컨드 단위입니다.
	int	m_delay;
    // 에니메이션인지 여부를 알려주는 값입니다.
	int m_animated;

    // 이미지의 폭입니다.
	int m_w;
    // 이미지의 높입니다.
	int m_h;
    // 이미지에 투명색 인덱스입니다. DecodeImage를 빠져나온후에는 더이상 사용할 수 없습니다.
	int m_trans;
	// colormap은 한시적으로 allocation되었다가 DecodeImage함수를 빠져 나간후에는 더 이상 사용할 수 없습니다.
	M_Uint32 *m_colormap;
    void *(*m_createImage)(struct _MC_GrpImgTarget *tgt, int w, int h, int plane);
    void (*m_setIndexPixels)(struct _MC_GrpImgTarget *tgt, 
            int x, int y, int w, int h,
            M_Uint8 *pxls, int bpl);
    void (*m_setRGBPixels)(struct _MC_GrpImgTarget *tgt, 
            int x, int y, int w, int h,
            M_Uint32 *pxls, int bpl);
	// 여기서 부터
    // 현재 읽어진 부분입니다.
	int current;
    // 이미지 소스의 전체 크기 입니다.
	int length;
	int offset;     // animation의 경우에 맨 앞으로 가야 하므로 offset이 필요
	// 여기 까지는 java array로 Image Object에 저장되는 내용입니다.
    // 이미지 소스의 버퍼 입니다.
	void *buf;
    int (*readBytes)(struct _MC_GrpImgSource *, M_Uint8 *, int sz);
    int (*readByte)(struct _MC_GrpImgSource *);
    int (*skipBytes)(struct _MC_GrpImgSource *, int n);
	int (*seekBytes)(struct _MC_GrpImgSource *, int n);
	int (*getPos)(struct _MC_GrpImgSource *);
} MC_GrpImgTarget;

/**
 * 이미지입니다.
 * 이미지는 내부에 프레임 버퍼와 기타 속성(애니메이션 여부등)가지고 있습니다.
 */
typedef M_Int32 MC_GrpImage;

/**
 * 애니메이션 여부의 이미지의 속성. 1로 정의되어있다.
 */
#define MC_GRP_IS_ANIMATED          1
/**
 * 애니메이션 지연 단위 이미지의 속성. 2로 정의되어있다.
 */
#define MC_GRP_ANIMATE_DELAY        2

/**
 * 애니메이션 루프 카운트 이미지의 속성. 3로 정의되어있다.
 */
#define MC_GRP_LOOP_COUNT           3

/**
 * 이미지의 넓이. 4로 정의되어있다.
 */
#define MC_GRP_IMAGE_WIDTH			4
/**
 * 이미지의 높이. 5로 정의되어있다.
 */
#define MC_GRP_IMAGE_HEIGHT			5
/**
 * 이미지의 BPP. 6로 정의되어있다.
 */
#define MC_GRP_IMAGE_BPP			6

/***********************************************************************************
		NetWork 관련 구조체 정의 
***********************************************************************************/


/// 인터넷 패밀리로 값은 2이다.
#define MC_AF_INET				2

#define MC_SOCKET_STREAM 	1
#define MC_SOCKET_DGRAM		2

typedef void (*NETCONNECTCB)(M_Int32 error, void *param);

typedef void (*NETSOCKCONNECTCB)(M_Int32 fd, M_Int32 error, void *param);

typedef void (*NETSOCKACCEPTCB)(M_Int32 sd, M_Int32 fd, M_Int32 error, void *param);

typedef void (*NETSOCKREADCB)(M_Int32 fd, M_Int32 error, void *param);

typedef void (*NETSOCKWRITECB)(M_Int32 fd, M_Int32 error, void *param);

typedef void (*NETHOSTADDRCB)(M_Int32 addr, void *param);

typedef void (*NETHTTPCB)(M_Int32 fd, M_Int32 sd, M_Int32 error, void *param);

/***********************************************************************************
		UTIL 관련 구조체 정의 
***********************************************************************************/
/**
    날짜와 시간에 대한 구조체이다.
 */
typedef struct MC_Date {
    /// Year [1980-2100]
    int year;
    /// Month of year [1-12]
    int month;
    /// Day of month [1-31] or day of year [1-366]
    int day;
    /// Hour of day [0-23]
    int hour;
    /// Minute of hour [0-59]
    int minute;
    /// Second of minute [0-59]
    int second;
    /// Day of the week [0-6] Monday-Sunday
    int day_of_week;
} MC_Date;

/***********************************************************************************
		시리얼 관련 구조체 정의 
***********************************************************************************/
typedef void (*SRLWRITECB)(M_Int32 fd, M_Int32 error, void *param);
typedef void (*SRLREADCB)(M_Int32 fd, M_Int32 error, void *param);

/***************************************************************
   C Widget
 ****************************************************************/
 typedef M_Int32 (*MC_UicEventHandlerProc)();
typedef void (*MC_UicCallbackProc)();
/**
 * 컴포넌트 식별자.
 */
typedef M_Int32 MC_UicComponent;

struct _MC_UicCallbackStr{
    M_Int32 idx;
    MC_UicCallbackProc proc;
    M_Int32 data;
};

/**
 * 컴포넌트 클래스 구조체 식별자.
 */


struct _MC_UicClass{
    M_Uint8 *name; 
    struct _MC_UicClass *super;
    M_Int32 size;
    M_Int32 (*create)(MC_UicComponent cmp);
    void (*destroy)(MC_UicComponent cmp);
    void (*paint)(MC_UicComponent cmp, MC_GrpContext *pgc);
    M_Uint32 (*handleEvent)(MC_UicComponent cmp, M_Int32 type, M_Int32 param1, M_Int32 param2);
};

typedef M_Int32 MC_UicClass;
//typedef struct _MC_UicClass *MC_UicClass;

/*
 */

struct _MC_UicComponent{
	MC_UicClass cls;

    //MC_UicClass cls;
	MC_GrpFrameBuffer screen;
    // 응용 프로그램
    M_Int32 x;
    M_Int32 y;
    M_Int32 w;
    M_Int32 h;
	M_Int32 mask;
	M_Int32 font;
	M_Int32 pxlfg;
	M_Int32 pxlbg;
    MC_UicEventHandlerProc handler;
    void *pcb;
    M_Int32 pcbSize;
	M_Int32 extdata;

};

/**
 * 응용 프로그램 컨택스트 식별자.
 */
typedef M_Int32 MC_UicApplicationContext; 

struct _MC_UicApplication{
	MC_GrpFrameBuffer screen; //screenID
};
/**
 * 콜백 함수 타입
 *
 * @param cc 콜백 함수를 호출하는 컴포넌트
 * @param serverData 부르는 쪽에서 넘기는 데이타; 호출하는 쪽마다 
 * 다릅니다.
 * @param clientData 콜백 함수를 
 */
typedef void (*MC_UicCallbackProc)(MC_UicComponent cc, 
        void *serverData, M_Int32 clientData);

/**
 * 이벤트 핸들러 함수 타입.
 *
 * @param cc 이벤트 핸들러를 호출하는 컴포넌트
 * @param type 이벤트 타입.
 * @param param1 이벤트 매개 변수1
 * @param param2 이벤트 매개 변수2
 * 
 * @return 이벤트 처리 여부 1이면 처리한 것이고, 0이면 처리하지 않음을 의미
 */
typedef M_Int32 (*MC_UicEventHandlerProc)(MC_UicComponent cc,
        M_Int32 type, M_Int32 param1, M_Int32 param2);

/**
 * 메뉴 컴포넌트 클래스의 문자열 "MenuComponent"로 정의 되어 있다.
 */
#define MC_UIC_MENU_COMPONENT     "MenuComponent"

/**
 * 데이트, 타임 컴포넌트 클래스의 문자열, "DateTimeComponent"로 정의 되어 있다.
 */
#define MC_UIC_DATE_TIME_COMPONENT     "DateTimeComponent"

/**
 * 텍스트 컴포넌트 클래스의 문자열
 */
#define MC_UIC_TEXT_COMPONENT     "TextComponent"

/**
 * 라벨 컴포넌트 클래스의 문자열
 */
#define MC_UIC_LABEL_COMPONENT    "LabelComponent"

/**
 * 리스트 컴포넌트 클래스의 문자열
 */
#define MC_UIC_LIST_COMPONENT       "ListComponent"

/**
 * 메모리 부족 오류 코드
 */
#define MC_UIC_E_OUT_OF_MEM         -1

/**
 * 컴포넌트가 소멸될 때 불리는 콜백 함수의 인덱스
 */
#define MC_UIC_DESTROY_CALLBACK         1
/**
 * 컴포넌트가 칠해질 때 불리는 콜백 함수의 인덱스
 */
#define MC_UIC_PAINT_CALLBACK           2
/**
 * 컴포넌트의 특정 내용이 선택될 때 불리는 콜백 함수의 인덱스
 */
#define MC_UIC_SELECT_CALLBACK          3
/**
 * 컴포넌트의 내부 내용이 변경될 때 불리는 콜백 함수의 인덱스
 */
#define MC_UIC_CHANGE_CALLBACK          4
/**
 * 컴포넌트에 사용자가 키를 눌렀을 때 불리는 콜백 함수의 인덱스
 */
#define MC_UIC_KEY_CALLBACK             5

#define MC_UIC_CALLBACK_END             5

/**
 * 흑백 타입
 */
#define MC_GRP_GRAY_TYPE       (1 << 1)

/**
 * 컬러 타입
 */
#define MC_GRP_COLOR_TYPE       (1 << 2)

/**
 *  콤포넌트의 크기와 위치를 지정하기위한 마스크
 */
#define MC_UIC_POS_MASK			(1 << 0)
#define MC_UIC_SIZE_MASK		(1 << 1)
/**
 *  콤포넌트의 상태를  지정하기위한 마스크
 */
#define MC_UIC_ENABLE_MASK		(1 << 2)

/* Label Component */

/* Label 콤포넌트의 정렬 방식 */
/**
 * 라벨컴포넌트의 좌측정렬
 */
#define MC_ALIGN_LEFT			0

/**
 * 라벨컴포넌트의 우측정렬
 */
#define MC_ALIGN_RIGHT			1

/**
 * 라벨컴포넌트의 중앙정렬
 */
#define MC_ALIGN_CENTER			2
#define MC_UIC_TIME_MASK        (1 << 0)
#define MC_UIC_DATE_MASK        (1 << 1)


/******************************************************************************/
/******         PHONE  API STRUCTURE **********************************************/
/******************************************************************************/

/**
  SMS 메시지의 상태를 변화시키는 명령어
 */
typedef enum MC_SmsCmd{
	MC_SMSCMD_READONLY = 0,	// 메시지 상태를 변경하지 않으면서 읽는 명령
	MC_SMSCMD_CHANGEREAD, // 메시지를 읽지 않은 상태에서 읽은 상태로 변환시키는 명령어
	MC_SMSCMD_DELETE, // 메시지를 삭제시키는 명령어
} MC_SmsCmd;


/**
  수신되는 SMS 데이터 를 가진 구조체
 */
typedef struct MC_SmsData{
	/// SMS Message의 번호
	M_Byte index; 
	/// 새로운 메시지 또는 읽었던 메시지 인지 구분. 0 이면 읽지 않은 메시지, 1 이면 읽은 메시지
	M_Byte class; 
	/// ASCII 문자열로 된 발신자 전화번호.<br>bar없이 붙여 사용된다. 예)016-123-4567 ->0161234567
	M_Byte callback[12];
	/// 발신자 전화번호의 길이.
	M_Byte cb_size; 
	/// 수신된 데이터
	M_Byte data[256];
	/// 수신된 데이터의 사이즈.
	M_Byte data_size; 
	/**
	  year 2byte,etc 1byte. 
	  <pre>
	  수신된 시간.
	  년	월	일	시	분	초
	  2 byte	1 byte	1 byte	1 byte	1 byte	1 byte
	  </pre>
	 */
	M_Byte timer[7];
} MC_SmsData;

/******************************************************************************/
/******         FILE  API STRUCTURE **********************************************/
/******************************************************************************/

/**
 * 파일 정부 구조체
 */
struct _fileInfo {
	///파일의 특성을 표시한 bit mask들
	M_Int32 attrib	;	
	///파일이 생성된 시간(초단위)
	M_Uint32 creationTime	;
	///파일의 크기
	M_Uint32 size	;	
};

/**
 * 파일 정보 구조체 타입
 */	
typedef struct _fileInfo MH_FileInfo;	

#define MH_FILE_OPEN_RDONLY         0x1		
#define MH_FILE_OPEN_WRONLY         0x2		
#define MH_FILE_OPEN_WRTRUNC        0x4		
#define MH_FILE_OPEN_RDWR           0x8		
#define MH_FILE_SEEK_SET			0	
#define MH_FILE_SEEK_CUR			1
#define MH_FILE_SEEK_END			2
#define MH_FILE_IS_DIR                          0x01


#define MC_FILE_OPEN_RDONLY		MH_FILE_OPEN_RDONLY
/// 쓰기만 가능
#define MC_FILE_OPEN_WRONLY		MH_FILE_OPEN_WRONLY
/// 쓰기만 가능하고 파일이 존재하면 파일 크기를 0으로 만듬
#define MC_FILE_OPEN_WRTRUNC	MH_FILE_OPEN_WRTRUNC
/// 읽기와 쓰기 모두 가능
#define MC_FILE_OPEN_RDWR		MH_FILE_OPEN_RDWR

/// 파일 attribute 중 디렉토리를 나타내는 bit
#define MC_FILE_IS_DIR 			MH_FILE_IS_DIR

//#define MH_MAX_FILENAME_LENGTH      30
/// 파일이름 최대 길이
#define MC_MAX_FILENAME_LENGTH		MH_MAX_FILENAME_LENGTH

/// 자기 자신의 디렉토리로 접근
#define MC_DIR_PRIVATE_ACCESS		1
/// 공유 디렉토리로 접근
#define MC_DIR_SHARED_ACCESS		2
/// 시스템 디렉토리로 접근
#define MC_DIR_SYSTEM_ACCESS		3

/// 파일의 처음을 기준으로 파일포인터의 위치를 설정
#define MC_FILE_SEEK_SET 	MH_FILE_SEEK_SET
/// 파일의 current position을 기준으로 파일포인터의 위치를 설정
#define MC_FILE_SEEK_CUR 	MH_FILE_SEEK_CUR
/// 파일의 끝을 기준으로 파일포인터의 위치를 설정
#define MC_FILE_SEEK_END 	MH_FILE_SEEK_END

typedef MH_FileInfo		MC_FileInfo;


/*********************************************************************/
/****     MULTI MEDIA ************************************************/
/*********************************************************************/
typedef void MC_MdaClip;
/**
	오류로 인한 정지 상태
	<P>
	값은 -1
 */
#define MC_MDA_STATUS_ERROR			(-1)
/**
	매체(혹은 톤)처리시 - 처리기가 매체(혹은 톤) 데이터의 마지막에 도달한 상태.
	
	<P>
	값은 1
 */
#define MC_MDA_STATUS_END_OF_DATA	1

/**
	매체(혹은 톤)처리시 - 매체(혹은 톤) 처리를 시작한 상태
	<P>
	값은 2
 */
#define MC_MDA_STATUS_STARTED		2

/**
	매체(혹은 톤)처리시 - 매체(혹은 톤) 처리를 종료한 상태
	<P>
	녹음시 - 녹음을 중단한 상태
	<P>
	값은 3
 */
#define MC_MDA_STATUS_STOPPED		3

/**
	매체(혹은 톤)처리시 - 매체(혹은 톤) 처리를 잠시 멈춘 상태
	<P>
	녹음시 - 녹음을 잠시 멈춘 상태
	<P>
	값은 4
 */
#define MC_MDA_STATUS_PAUSED		4

/**
	매체(혹은 톤)처리시 - 잠시 멈춘 매체(혹은 톤) 처리를 재개한 상태
	<P>
	녹음시 - 잠시 멈춘 녹음을 재개한 상태
	<P>
	값은 5
 */
#define MC_MDA_STATUS_RESUMED		5

/**
	녹음시 - 녹음을 시작한 상태
	<P>
	값은 6
 */
#define MC_MDA_STATUS_RECORDED		6

/**
	녹음시 - 클립내부버퍼가 완전히 채워진 상태
	<P>
	값은 7
 */
#define MC_MDA_STATUS_FULL_OF_DATA		7

/**
	TONE TYPE의  열거형 소리의 음계를 나타낸다.
 */
typedef enum MC_MdaToneType {
/// DTMF for 0 key
MC_SND_TONE_0 = 0,
/// DTMF for 1 key 		
MC_SND_TONE_1, 	
/// DTMF for 2 key		
MC_SND_TONE_2,
/// DTMF for 3 key 			
MC_SND_TONE_3,
/// DTMF for 4 key	 		
MC_SND_TONE_4,
/// DTMF for 5 key 			
MC_SND_TONE_5,
/// DTMF for 6 key
MC_SND_TONE_6,
/// DTMF for 7 key
MC_SND_TONE_7, 	
/// DTMF for 8 key		
MC_SND_TONE_8, 	
/// DTMF for 9 key
MC_SND_TONE_9, 		
/// DTMF for A key
MC_SND_TONE_A, 		
/// DTMF for B key
MC_SND_TONE_B, 		
/// DTMF for C key
MC_SND_TONE_C, 		
/// DTMF for D key
MC_SND_TONE_D, 		
/// DTMF for # key
MC_SND_TONE_POUND, 	
/// DTMF for * key
MC_SND_TONE_STAR, 	
///  440.0 Hz  -Piano Notes-
MC_SND_NOTE_A4,	 
///  466.1 Hz	
MC_SND_NOTE_AS4,
///  493.8 Hz 
MC_SND_NOTE_B4, 
///  523.2 Hz
MC_SND_NOTE_C4, 
///  554.3 Hz
MC_SND_NOTE_CS4,
///  587.3 Hz
MC_SND_NOTE_D4, 
///  622.2 Hz
MC_SND_NOTE_DS4,
///  659.2 Hz 
MC_SND_NOTE_E4, 
///  698.5 Hz
MC_SND_NOTE_F4, 
///  739.9 Hz
MC_SND_NOTE_FS4,
///  784.0 Hz 
MC_SND_NOTE_G4, 
///  830.6 Hz
MC_SND_NOTE_GS4,
///  880.0 Hz 
MC_SND_NOTE_A5, 
///  932.2 Hz
MC_SND_NOTE_AS5,
///  987.7 Hz 
MC_SND_NOTE_B5, 
/// 1046.5 Hz
MC_SND_NOTE_C5, 
/// 1108.7 Hz
MC_SND_NOTE_CS5,
/// 1174.6 Hz 
MC_SND_NOTE_D5, 
/// 1244.3 Hz
MC_SND_NOTE_DS5,
/// 1318.5 Hz 
MC_SND_NOTE_E5, 
/// 1397.0 Hz
MC_SND_NOTE_F5, 
/// 1479.9 Hz
MC_SND_NOTE_FS5,
/// 1568.0 Hz 
MC_SND_NOTE_G5, 
/// 1661.2 Hz
MC_SND_NOTE_GS5,
/// 1760.0 Hz 
MC_SND_NOTE_A6, 
/// 1864.7 Hz
MC_SND_NOTE_AS6,
/// 1975.5 Hz 
MC_SND_NOTE_B6, 
/// 2093.1 Hz
MC_SND_NOTE_C6, 
/// 2217.4 Hz
MC_SND_NOTE_CS6,
/// 2349.3 Hz 
MC_SND_NOTE_D6, 
/// 2489.1 Hz
MC_SND_NOTE_DS6, 
/// 2637.0 Hz
MC_SND_NOTE_E6,
/// 2793.7 Hz
MC_SND_NOTE_F6, 
/// 2959.9 Hz
MC_SND_NOTE_FS6,
/// 3135.9 Hz 
MC_SND_NOTE_G6, 
/// 3322.4 Hz
MC_SND_NOTE_GS6,
/// 3520.0 Hz 
MC_SND_NOTE_A7
} MC_MdaToneType; 

/**
	톤 재생시 배열로 전달될 Tone 구조체. 
 */
typedef struct MC_MdaTone {
	/// 연주될 TONE
	MC_MdaToneType  tone;
	/// 연주시간(ms)
	M_Int32 	duration; 	
} MC_MdaTone; 

/**
	프리퀀시 톤 재생시 배열로 전달될 Tone 구조체. 
 */
typedef struct MC_MdaFreqTone {
	/// 연주될 톤의 고주파 HZ
	M_Int32 hiFreq;
	/// 연주될 톤의 저주파 HZ
	M_Int32 lowFreq;	
	/// 연주시간(ms)	
	M_Int32 	duration; 	
} MC_MdaFreqTone; 

typedef void MC_MdaClip;
/**
	현재 볼륨값을 의미하는 상수
	<P>
	값은 0
 */
#define MC_MDA_VOLSEL_CUR	0

/**
	최소 볼륨값을 의미하는 상수
	<P>
	값은 1
 */
#define MC_MDA_VOLSEL_MIN	1

/**
	최대 볼륨값을 의미하는 상수
	<P>
	값은 2
 */
#define MC_MDA_VOLSEL_MAX	2

/**
	볼륨 소스가 톤임을 의미하는 상수
	<P>
	값은 0
 */
#define MC_MDA_VOLTYPE_TONE		0

/**
	볼륨 소스가 오디오임을 의미하는 상수
	<P>
	값은 1
 */
#define MC_MDA_VOLTYPE_SOUND	1
/**
	볼륨 소스가 녹음기임을 의미하는 상수
	<P>
	값은 2
 */
#define MC_MDA_VOLTYPE_RECORDER	2

typedef enum MC_BackLight {
	///백라이트를 켬
	MC_LIGHT_ON = 0,
	///백라이트를 끔
	MC_LIGHT_OFF,
	///백라이트를 항상 켬
	MC_LIGHT_ALWAYS_ON,
	///사용자가 설정한 상태로 둠
	MC_LIGHT_DEFAULT
} MC_BackLight;;

/**
	처리기 상태가 변경될 때 불려지는 함수이다. 상태 값은 매체처리 상태참조
	@param clip 클립
	@param status 매체처리 상태
 */
typedef void (*MEDIACB)(MC_MdaClip* clip, M_Int32 status);


typedef M_Uint32 jclass;

/*
typedef struct _MExInterface {
	jclass (*MNI_getObjectClass)(jobject obj);
	M_Boolean (*MNI_instanceof)(jclass cs, jobject obj);
	void (*MNI_javaString2LocalCode)(jobject jStrObj, char* cs, int len);
	jobject (*MNI_localCode2JavaString)(char* cStr, int len);

	void* (*MNI_getModuleEntryPointer)(M_Char* moduleName, M_Int32 major, M_Int32 minor);
	void (*MNI_setSecurity)(M_Int32 security);
} MExInterface;
*/
typedef struct _MExInterface {
	void* (*getModuleInterface)(M_Char* moduleName, M_Int32 major, M_Int32 minor);
} MExInterface;

typedef struct _MSecureExInterface {
	void (*dummyFunc)();
} MSecureExInterface;

typedef struct _MG_NativeEnv {
	int32** Java_sp;

/************************************************************
     MNI 관련 API 정의
 ************************************************************/
 	void (*E_MNI_raiseExceptionObj)(jobject eObj, char* msg);
 	void (*E_MNI_raiseException)(char* excpClassName, char* msg);
 	M_Uint32 (*E_MNI_getPSpaceBase)();
 	M_Boolean (*E_MNI_setRewindPSpace)(M_Uint32 pSpaceBase);
	jobject (*E_MNI_makeJavaString)(char* cStr, int len);
	M_Int32 (*E_MNI_getArrayLength)( jobject arr );
	void* (*E_MNI_pAlloc)(M_Uint32 size);
	void* (*E_MNI_pCalloc)(M_Uint32 size);
	jobject (*E_MNI_newArrayObject)(jclass arrCs, jint numbers);
	M_Char* (*E_MNI_javaString2CString)(jobject jStrObj, char* cBuf, int bufLen);
	M_Char* (*E_MNI_loadClass)(M_Uint32* clss, char* className);
	jobject (*E_MNI_newObject)(jclass cs);
	MExInterface* (*E_MNI_getExInterface)();
	MSecureExInterface* (*E_MNI_getSecureExInterface)();
/************************************************************/

	void (*E_MC_knlPrintk)(M_Char* fmt, ...);
	void (*E_MC_knlSprintk)(M_Char* buf, M_Char* format, ...);
	M_Int32 (*E_MC_knlExecute)(M_Char* execName, M_Int32 parmCnt, ...);
	M_Int32 (*E_MC_knlMExecute)(M_Char* symName, M_Int32 parmCnt, ...);
	M_Int32 (*E_MC_knlLoad)(M_Char* execName, M_Int32 parmCnt, ...);
	M_Int32 (*E_MC_knlMLoad)(M_Char* symName, M_Int32 parmCnt, ...);
	void (*E_MC_knlExit)(M_Int32 exitCode);
	M_Int32 (*E_MC_knlProgramStop)(M_Int32 prgID);
	M_Int32 (*E_MC_knlGetExecNames)(M_Char* prgName, M_Char* version, M_Char* vendor, M_Char* rtnBuf, M_Int32 bufSize);
	void (*E_MC_knlDefTimer)(MCTimer* tm, void* tcb);
	M_Int32 (*E_MC_knlSetTimer)(MCTimer* tm, M_Int64 timeout, void* parm);
	void (*E_MC_knlUnsetTimer)(MCTimer* tm);
	void* (*E_MC_knlCreateSharedBuf)(M_Char* name, M_Int32 size);
	void* (*E_MC_knlGetSharedBuf)();
	void* (*E_MC_knlResizeSharedBuf)(M_Int32 resize);
	M_Int32 (*E_MC_knlGetSharedBufSize)();
	M_Uint32 (*E_MC_knlCalloc)(M_Int32 size);
	void (*E_MC_knlFree)(M_Uint32 mID);
	M_Int32 (*E_MC_knlGetTotalMemory)();
	M_Int32 (*E_MC_knlGetFreeMemory)();
	M_Int32 (*E_MC_knlGetCurProgramID)();
	M_Int32 (*E_MC_knlGetParentProgramID)();
	M_Int32 (*E_MC_knlGetAppManagerID)();
	M_Int32 (*E_MC_knlGetProgramInfo)(M_Int32* buf, M_Int32 bufSize);


	M_Int32 (*E_MC_knlGetAccessLevel)();
	M_Int32 (*E_MC_knlGetProgramName)(M_Char* nameBuf, M_Int32 bufSize);
	M_Int64 (*E_MC_knlCurrentTime)();
	M_Int32 (*E_MC_knlGetSystemProperty)(M_Char* command, M_Char* buf, M_Int32 bufSize);
	M_Int32 (*E_MC_knlSetSystemProperty)(M_Char* id, M_Char* buf);
	M_Int32 (*E_MC_knlGetResourceID)(M_Char* rName, M_Int32* size);
	M_Int32 (*E_MC_knlGetResource)(M_Int32 rsID, void* ibuf, M_Int32 size);

	M_Int32 (*E_MC_grpGetImageProperty)(MC_GrpImage img, int index);
	M_Int32 (*E_MC_grpGetImageFrameBuffer)(MC_GrpImage img);
	M_Int32 (*E_MC_grpGetScreenFrameBuffer)(int i);
	void 	(*E_MC_grpDestroyOffScreenFrameBuffer)(MC_GrpFrameBuffer fb);
	M_Int32 (*E_MC_grpCreateOffScreenFrameBuffer)(M_Int32 w, M_Int32 h);
	void 	(*E_MC_grpInitContext)(MC_GrpContext *pgc);
	void 	(*E_MC_grpSetContext)(MC_GrpContext *pgc, M_Int32 index, void *pv);
	void 	(*E_MC_grpGetContext)(MC_GrpContext *pgc, M_Int32 index, void *pv);
	void 	(*E_MC_grpPutPixel)(MC_GrpFrameBuffer dst,M_Int32 x, M_Int32 y, MC_GrpContext *pgc);
	void 	(*E_MC_grpDrawLine)(MC_GrpFrameBuffer dst,M_Int32 x1, M_Int32 y1, M_Int32 x2, M_Int32 y2,MC_GrpContext *pgc);
	void 	(*E_MC_grpDrawRect)(MC_GrpFrameBuffer dst,  M_Int32 x, M_Int32 y,  M_Int32 w, M_Int32 h,    MC_GrpContext *pgc);
	void 	(*E_MC_grpFillRect)(MC_GrpFrameBuffer dst,  M_Int32 x, M_Int32 y,  M_Int32 w, M_Int32 h,  MC_GrpContext *pgc);
	void 	(*E_MC_grpCopyFrameBuffer)(MC_GrpFrameBuffer dst,   M_Int32 dx, M_Int32 dy, M_Int32 w, M_Int32 h,  MC_GrpFrameBuffer src,  M_Int32 sx, M_Int32 sy, MC_GrpContext *pgc);
	void 	(*E_MC_grpDrawImage)(MC_GrpFrameBuffer dst,  M_Int32 dx, M_Int32 dy, M_Int32 w, M_Int32 h,  MC_GrpImage src,  M_Int32 sx, M_Int32 sy, MC_GrpContext *pgc);
	void 	(*E_MC_grpCopyArea)(MC_GrpFrameBuffer dst, M_Int32 dx, M_Int32 dy, M_Int32 w, M_Int32 h, M_Int32 x, M_Int32 y, MC_GrpContext *pgc);
	void 	(*E_MC_grpDrawArc)(MC_GrpFrameBuffer dst, M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h, M_Int32 s, M_Int32 e, MC_GrpContext *pgc);
	void 	(*E_MC_grpFillArc)(MC_GrpFrameBuffer dst, M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h, M_Int32 s, M_Int32 e, MC_GrpContext *pgc);
	void 	(*E_MC_grpDrawString)(MC_GrpFrameBuffer dst, M_Int32 x, M_Int32 y,  const char *str, M_Int32 len, MC_GrpContext *pgc);
	void 	(*E_MC_grpDrawUnicodeString)(MC_GrpFrameBuffer dst, M_Int32 x, M_Int32 y, const M_UCode *str, M_Int32 len, MC_GrpContext *pgc);
	void 	(*E_MC_grpGetRGBPixels)(MC_GrpFrameBuffer dst,  M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h, M_Uint32 *pd, M_Int32 ipl);
	void 	(*E_MC_grpSetRGBPixels)(MC_GrpFrameBuffer dst, M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h, const M_Uint32 *psrc, M_Int32 ibpl, MC_GrpContext *pgc);
	void 	(*E_MC_grpFlushLcd)(M_Int32 i,  MC_GrpFrameBuffer frm, M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h);
	M_Int32 (*E_MC_grpGetPixelFromRGB)(M_Int32 r, M_Int32 g, M_Int32 b);

	M_Int32 (*E_MC_grpGetDisplayInfo)(M_Int32 i, MC_GrpDisplayInfo *pi);
	void 	(*E_MC_grpRepaint)(M_Int32 lcd, M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h);
	M_Int32 (*E_MC_grpGetFont)(M_Int32 face, M_Int32 size, M_Int32 style);
	M_Int32 (*E_MC_grpGetFontHeight)(M_Int32 font);
	M_Int32 (*E_MC_grpGetFontAscent)(M_Int32 font);
	M_Int32 (*E_MC_grpGetFontDescent)(M_Int32 font);
	M_Int32 (*E_MC_grpGetStringWidth)(M_Int32 font, const M_Uint8 *str, M_Int32 len);
	M_Int32 (*E_MC_grpGetUnicodeStringWidth)(M_Int32 font, const M_UCode *str, M_Int32 len);
	M_Int32 (*E_MC_grpCreateImage)(MC_GrpImage *newImg, M_Int32 bufID, M_Int32 off, M_Int32 len);
	void 	(*E_MC_grpDestroyImage)(MC_GrpImage img);
	M_Int32 (*E_MC_grpDecodeNextImage)(MC_GrpImage dst);
	M_Int32 (*E_MC_grpEncodeImage)(MC_GrpFrameBuffer src, M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h, M_Int32 *len);
	M_Int32 (*E_MC_grpPostEvent)(M_Int32 id, M_Int32 type, M_Int32 param1, M_Int32 param2);

	M_Int32 (*E_MC_imGetSurpportModeCount)();
	char**  (*E_MC_imGetSupportedModes)();
	M_Int32 (*E_MC_imSetCurrentMode)(M_Int32 mode);
	M_Int32 (*E_MC_imGetCurrentMode)();
	M_Int32 (*E_MC_imHandleInput)(char key, M_Int32 type,char *buf1,M_Int32 *size1,char *buf2,M_Int32 *size2);

	MC_UicApplicationContext (*E_MC_uicCreateApplicationContext)();
	MC_UicClass (*E_MC_uicGetClass)(M_Uint8 *psz);
	MC_UicComponent (*E_MC_uicCreate)(MC_UicApplicationContext *pac, MC_UicClass *cls);
	void (*E_MC_uicDestroy)(MC_UicComponent cc);
	void (*E_MC_uicRepaint)(MC_UicComponent cc,  M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h);
	void (*E_MC_uicPaint)(MC_UicComponent cc, MC_GrpContext *pgc);
	M_Uint8* (*E_MC_uicGetClassName)(MC_UicComponent cc);
	M_Uint32 (*E_MC_uicIsInstance)(MC_UicComponent cc, M_Uint8 *pcls); 
	M_Int32 (*E_MC_uicHandleEvent)(MC_UicComponent cc, M_Int32 type, M_Int32 param1, M_Int32 param2);
	void (*E_MC_uicConfigure)(MC_UicComponent cc, M_Int32 x, M_Int32 y, M_Int32 w, M_Int32 h, M_Int32 mask);
	void (*E_MC_uicGetGeometry)(MC_UicComponent cc, M_Int32 *px, M_Int32 *py, M_Int32 *pw, M_Int32 *ph);
	M_Int32 (*E_MC_uicSetEnable)(MC_UicComponent cc, M_Int32 enable);
	M_Int32 (*E_MC_uicSetExtData)(MC_UicComponent cc, M_Int32 data);
	M_Int32 (*E_MC_uicGetExtData)(MC_UicComponent cc);
	MC_UicCallbackProc (*E_MC_uicSetCallback)( MC_UicComponent cc, M_Int32 idx, MC_UicCallbackProc proc, M_Int32 clientData);
	MC_UicEventHandlerProc (*E_MC_uicSetEventHandler)( MC_UicComponent cc, MC_UicEventHandlerProc handler);
	M_Int32 (*E_MC_uicSetFont)(MC_UicComponent cc, M_Int32 fontid);
	M_Int32 (*E_MC_uicGetFont)(MC_UicComponent cc);
	void (*E_MC_uicSetFgColor)(MC_UicComponent cc, M_Int32 nColor);
	void (*E_MC_uicSetBgColor)(MC_UicComponent cc, M_Int32 nColor);
	void (*E_MC_uicSetLabel)(MC_UicComponent cc, M_Uint8 *psz);
	M_Uint8* (*E_MC_uicGetLabel)(MC_UicComponent cc);
	M_Int32 (*E_MC_uicSetLabelAlignment)(MC_UicComponent cc, M_Int32 align);
	M_Int32 (*E_MC_uicSetTimeMask)(MC_UicComponent cc, M_Int32 mask);
	void (*E_MC_uicSetTime)(MC_UicComponent cc, struct tm *pTM);
	void (*E_MC_uicSetTimeLong)(MC_UicComponent cc, time_t time);
	void (*E_MC_uicGetTime)(MC_UicComponent cc, struct tm *pTM);
	M_Int32 (*E_MC_uicAddMenuItem)(MC_UicComponent cc, M_Uint8 *psz, MC_GrpImage img);
	M_Int32 (*E_MC_uicGetMenuItem)(MC_UicComponent cc, M_Uint32 idx,  M_Uint8 *psz, M_Int32 buflen, MC_GrpImage *img);
	M_Int32 (*E_MC_uicRemoveMenuItem)(MC_UicComponent cc, M_Uint32 idx);
	M_Int32 (*E_MC_uicSetActiveMenuItem)(MC_UicComponent cc, M_Int32 idx);
	M_Int32 (*E_MC_uicGetActiveMenuItem)(MC_UicComponent cc);
	M_Int32 (*E_MC_uicInsertText)(MC_UicComponent cc, M_Int32 idx, M_Uint8 *psz, M_Int32 len);
	void (*E_MC_uicDeleteText)(MC_UicComponent cc, M_Int32 idx, M_Int32 len);
	M_Int32 (*E_MC_uicGetMaxTextSize)(MC_UicComponent cc);
	M_Int32 (*E_MC_uicSetMaxTextSize)(MC_UicComponent cc, M_Int32 max);
	M_Int32 (*E_MC_uicGetTextSize)(MC_UicComponent cc);
	M_Int32 (*E_MC_uicGetText)(MC_UicComponent cc, M_Int32 idx, M_Uint8 *pszBuf, M_Int32 len);
	M_Int32 (*E_MC_uicAddListItem)(MC_UicComponent cc, M_Uint8 *psz, MC_GrpImage img);
	M_Int32 (*E_MC_uicGetListItem)(MC_UicComponent cc, M_Uint32 idx,  M_Uint8 *psz, M_Int32 buflen, MC_GrpImage *img);
	M_Int32 (*E_MC_uicRemoveListItem)(MC_UicComponent cc, M_Uint32 idx);
	M_Int32 (*E_MC_uicSetActiveListItem)(MC_UicComponent cc, M_Int32 idx);
	M_Int32 (*E_MC_uicGetActiveListItem)(MC_UicComponent cc);
	
// 네트웍 접근
	M_Int32 (*E_MC_netConnect)( NETCONNECTCB cb, void *param);
	void (*E_MC_netClose)();
// 네트웍 소켓
	M_Int32 (*E_MC_netSocket)(M_Int32 domain, M_Int32 type);
	M_Int32 (*E_MC_netSocketBind)(M_Int32 fd, M_Int32 addr, M_Uint16 port);
	M_Int32 (*E_MC_netSocketConnect)(M_Int32 fd, M_Int32 addr, M_Int16 port, NETSOCKCONNECTCB cb, void *param);
	M_Int32 (*E_MC_netSocketWrite)(M_Int32 fd, M_Byte* buf, M_Int32 len);
	M_Int32 (*E_MC_netSocketRead)(M_Int32 fd, M_Byte* buf, M_Int32 len);
	M_Int32 (*E_MC_netSocketSendTo)(M_Int32 fd, M_Byte* buf, M_Int32 len, M_Uint32 addr, M_Uint16  port);
	M_Int32 (*E_MC_netSocketRcvFrom)(M_Int32 fd, M_Byte* buf, M_Int32 len, M_Uint32* addr, M_Uint16* port);
	M_Int32 (*E_MC_netSetWriteCB)(M_Int32 fd, NETSOCKWRITECB cb, void *param);
	M_Int32 (*E_MC_netSetReadCB)(M_Int32 fd, NETSOCKREADCB cb, void *param);
	M_Int32 (*E_MC_netSocketClose)(M_Int32 fd);
	M_Int32 (*E_MC_netGetMaxPacketLength)(void);
	M_Int32 (*E_MC_netGetHostAddr)(M_Int32 dnsserver, M_Byte *hostname, NETHOSTADDRCB cb, void *param);
	M_Int32 (*E_MC_netSocketAccept)(M_Int32 fd, NETSOCKACCEPTCB cb, void *param);
// 네트웍 HTTP
	M_Int32 (*E_MC_netHttpOpen)(M_Byte* url);
	M_Int32 (*E_MC_netHttpConnect)(M_Int32 fd, NETHTTPCB cb, void *param);
	M_Int32 (*E_MC_netHttpSetRequestMethod)(M_Byte *method);
	M_Int32 (*E_MC_netHttpGetRequestMethod)(M_Int32 fd, M_Byte *buf, M_Int32 len);
	M_Int32 (*E_MC_netHttpSetRequestProperty)(M_Byte *key, M_Byte *value);
	M_Int32 (*E_MC_netHttpGetRequestProperty)(M_Byte *buf, M_Int32 len);
	M_Int32 (*E_MC_netHttpSetProxy)(M_Int32 proxyhost, M_Int16 proxyport);
	M_Int32 (*E_MC_netHttpGetProxy)(M_Int32 fd, M_Int32 *proxyhost, M_Int16 *proxyport);
	M_Int32 (*E_MC_netHttpGetResponseCode)();
	M_Int32 (*E_MC_netHttpGetResponseMessage)(M_Byte *buf, M_Int32 len);
	M_Int32 (*E_MC_netHttpGetHeaderField)(M_Byte *name, M_Byte *buf, M_Int32 len);
	M_Int32 (*E_MC_netHttpGetLength)();
	M_Int32 (*E_MC_netHttpGetType)(M_Byte *buf, M_Int32 len);
	M_Int32 (*E_MC_netHttpGetEncoding)(M_Byte *buf, M_Int32 len);
	M_Int32 (*E_MC_netHttpClose)(M_Int32 fd);
	M_Int32 (*E_MC_netHandleEvent)(M_Int32 fd, M_Int32 event);
// 시리얼
	M_Int32 (*E_MC_srlOpen)(M_Int32 port, M_Byte* param);
	M_Int32 (*E_MC_srlWrite)(M_Int32 fd, M_Uint8* buf, M_Int32 size);
	M_Int32 (*E_MC_srlRead)(M_Int32 fd, M_Uint8* buf, M_Int32 size);
	M_Int32 (*E_MC_srlSetReadCB)(M_Int32 fd, SRLREADCB cb, void *param);
	M_Int32 (*E_MC_srlSetWriteCB)(M_Int32 fd, SRLWRITECB cb, void *param);
	M_Int32 (*E_MC_srlClose)(M_Int32 fd);
	void	(*E_MC_srlHandleEvent)(M_Int32 fd, M_Int32 event);
// 파일
	M_Int32 (*E_MC_fsOpen)(char* name, M_Int32 flag, M_Int32 aMode);
	M_Int32 (*E_MC_fsRead)(M_Int32 fd, char* buf, M_Int32 len);
	M_Int32 (*E_MC_fsWrite)(M_Int32 fd, char* buf, M_Int32 len);
	M_Int32 (*E_MC_fsClose)(M_Int32 fd);
	M_Int32 (*E_MC_fsSeek)(M_Int32 fd, M_Int32 pos, int where);
	M_Int32 (*E_MC_fsFileAttribute)(char* name, MC_FileInfo* fa, M_Int32 aMode);
	M_Int32 (*E_MC_fsRemove)(char* name, M_Int32 aMode);
	M_Int32 (*E_MC_fsRename)(char* oldname, char* newname, M_Int32 aMode);
	M_Int32 (*E_MC_fsMkDir)(char* dirName, M_Int32 aMode);
	M_Int32 (*E_MC_fsRmDir)(char* dirName, M_Int32 aMode);
	M_Int32 (*E_MC_fsList)(char *name,  char* buf, M_Int32 bufSize, M_Int32 aMode);
	M_Int32 (*E_MC_fsTotalSpace)(void);
	M_Int32 (*E_MC_fsAvailable)(void);
//MDA
	MC_MdaClip* (*E_MC_mdaClipCreate)(M_Char* mType, M_Int32 bufSize, MEDIACB cb);
	M_Int32 (*E_MC_mdaClipFree)(MC_MdaClip* clip);
	void (*E_MC_mdaSetWaterMark)(MC_MdaClip* clip, M_Int32 percent) ;
	M_Int32 (*E_MC_mdaClipGetType)(MC_MdaClip* clip, M_Byte* buf, M_Int32 bufSize);
	M_Int32 (*E_MC_mdaClipPutData)(MC_MdaClip* clip, M_Byte* buf, M_Int32 size);
	M_Int32 (*E_MC_mdaClipPutToneData)(MC_MdaClip* clip, MC_MdaToneType* tone, M_Int32* duration,  M_Int32 number);
	M_Int32 (*E_MC_mdaClipPutFreqToneData)(MC_MdaClip* clip, M_Int32 hiFreq[], M_Int32 lowFreq[], M_Int32 duration, M_Int32 number);
	M_Int32 (*E_MC_mdaClipGetData)(MC_MdaClip* clip, M_Byte* buf, M_Int32 size);
	M_Int32 (*E_MC_mdaClipAvailableDataSize)(MC_MdaClip* clip);
	void (*E_MC_mdaClipClearData)(MC_MdaClip* clip);
	M_Int32 (*E_MC_mdaPlay)(MC_MdaClip* clip, M_Boolean repeat);
	M_Int32 (*E_MC_mdaPause)(MC_MdaClip* clip);
	M_Int32 (*E_MC_mdaResume)(MC_MdaClip* clip);
	M_Int32 (*E_MC_mdaStop)(MC_MdaClip* clip);
	M_Int32 (*E_MC_mdaRecord)(MC_MdaClip* clip);
	M_Int32 (*E_MC_mdaGetVolume)();
	void (*E_MC_mdaSetVolume)(M_Int32 value);
	void (*E_MC_mdaVibrator)(M_Int32 on_off, M_Int32 timeout);
// DB
    M_Int32 (*E_MC_dbOpenDataBase)(char *name, M_Int32 rsize,
            M_Boolean create, M_Int32 mode);
    M_Int32 (*E_MC_dbCloseDataBase)(M_Int32 fd);
    M_Int32 (*E_MC_dbDeleteDataBase)(char *dataBaseName, M_Int32 mode);
    M_Int32 (*E_MC_dbInsertRecord)(M_Int32 fd, char *buf, M_Int32 len);
    M_Int32 (*E_MC_dbSelectRecord)(M_Int32 fd, M_Int32 recid,
            char *buf, M_Int32 len);
    M_Int32 (*E_MC_dbUpdateRecord)(M_Int32 fd, M_Int32 recid,
            char *buf, M_Int32 len);
    M_Int32 (*E_MC_dbDeleteRecord)(M_Int32 fd, M_Int32 recId);
    M_Int32 (*E_MC_dbListRecords)(M_Int32 fd, M_Int32 *buf, M_Int32 len);
    M_Int32 (*E_MC_dbSortRecords)(M_Int32 fd, M_Int32 *buf, M_Int32 len,
            M_Int32 (*compare)(const void *, const void*),
            M_Int32 (*filter)(const void*));
    M_Int32 (*E_MC_dbGetAccessMode)(char *dataBaseName);
    M_Int32 (*E_MC_dbGetNumberOfRecords)(M_Int32 fd);
    M_Int32 (*E_MC_dbGetRecordSize)(M_Int32 fd);
    M_Int32 (*E_MC_dbListDataBases)(char *buf, M_Int32 size);

// util
M_Int32 (*E_MC_utilGetResource)(M_Char* resourceName,M_Int32* size);
	//void 	(*E_MC_utilGetDate)(MC_Date* d);
	//M_Int32 (*E_MC_utilComputeDate)(MC_Date* d);
	M_Int32 (*E_MC_utilHtonl)(M_Int32 val);
	M_Int16 (*E_MC_utilHtons)(M_Int16 val);
	M_Int32 (*E_MC_utilNtohl)(M_Int32 val);
	M_Int16 (*E_MC_utilNtohs)(M_Int16 va);
	M_Int32 (*E_MC_utilInetAddrInt)(M_Byte *addr);
	void 	(*E_MC_utilInetAddrStr)(M_Int32 ip, M_Byte* addr);

// phone
	M_Int32 (*E_MC_phnCallPlace)(M_Byte* phonenumber);
	M_Int32 (*E_MC_phnSmsOpen)(M_Byte* telIDString, MC_SmsCmd cmd);
	M_Int32 (*E_MC_phnGetSMSAvailable)(M_Int32 fd);
	M_Int32 (*E_MC_phnSmsClose)(M_Int32 fd);
	M_Int32 (*E_MC_phnSmsRead)(M_Int32 fd, MC_SmsData* buf);
	M_Int32 (*E_MC_phnSmsSend)(M_Byte* telIDString, M_Char * telNum, M_Byte* buf, M_Int32 len);
//나중에 추가 된것	
    M_Int32 (*E_MC_grpGetRGBFromPixel)(M_Int32 pixel, M_Int32 *r, M_Int32 *g, M_Int32 *b);



	// WIPI C misc func
	M_Int32 (*E_MC_miscBackLight)(M_Int32 id, MC_BackLight on_off, M_Int32 color, M_Int32 timeout);
	void (*E_MC_miscSetLed)(M_Int32 leds);
	M_Int32 (*E_MC_miscGetLed)();
	M_Int32 (*E_MC_miscGetLedCount)(void);

	M_Int32 (*E_MC_knlDestroySharedBuf)(void* bufID);

} MG_NativeEnv;

extern MG_NativeEnv* nEnv;

#define MNI_raiseExceptionObj nEnv->E_MNI_raiseExceptionObj
#define MNI_raiseException nEnv->E_MNI_raiseException
#define MNI_getPSpaceBase nEnv->E_MNI_getPSpaceBase
#define MNI_setRewindPSpace nEnv->E_MNI_setRewindPSpace
#define MNI_makeJavaString	nEnv->E_MNI_makeJavaString
#define MNI_getArrayLength	nEnv->E_MNI_getArrayLength
#define MNI_pAlloc		nEnv->E_MNI_pAlloc
#define MNI_pCalloc		nEnv->E_MNI_pCalloc
#define MNI_newArrayObject	nEnv->E_MNI_newArrayObject
#define MNI_javaString2CString	  nEnv->E_MNI_javaString2CString
#define MNI_loadClass	nEnv->E_MNI_loadClass
#define MNI_newObject	nEnv->E_MNI_newObject
#define MNI_getExInterface	nEnv->E_MNI_getExInterface
#define MNI_getSecureExInterface nEnv->E_MNI_getSecureExInterface

#define MC_knlPrintk	 nEnv->E_MC_knlPrintk 
#define MC_knlSprintk	 nEnv->E_MC_knlSprintk
#define MC_knlExecute 	nEnv->E_MC_knlExecute
#define MC_knlMExecute	 nEnv->E_MC_knlMExecute
#define MC_knlLoad		 nEnv->E_MC_knlLoad
#define MC_knlMLoad 	nEnv->E_MC_knlMLoad
#define MC_knlExit 		nEnv->E_MC_knlExit
#define MC_knlProgramStop	 nEnv->E_MC_knlProgramStop
#define MC_knlGetExecNames 	nEnv->E_MC_knlGetExecNames
#define MC_knlDefTimer 	nEnv->E_MC_knlDefTimer
#define MC_knlSetTimer 	nEnv->E_MC_knlSetTimer
#define MC_knlUnsetTimer 	nEnv->E_MC_knlUnsetTimer
#define MC_knlCreateSharedBuf 	nEnv->E_MC_knlCreateSharedBuf
#define MC_knlGetSharedBuf 	nEnv->E_MC_knlGetSharedBuf
#define MC_knlResizeSharedBuf 	nEnv->E_MC_knlResizeSharedBuf
#define MC_knlGetSharedBufSize 	nEnv->E_MC_knlGetSharedBufSize
#define MC_knlAlloc 	nEnv->E_MC_knlCalloc
#define MC_knlCalloc 	nEnv->E_MC_knlCalloc
#define MC_knlFree 	nEnv->E_MC_knlFree
#define MC_knlGetTotalMemory 	nEnv->E_MC_knlGetTotalMemory
#define MC_knlGetFreeMemory 	nEnv->E_MC_knlGetFreeMemory
#define MC_knlGetCurProgramID 	nEnv->E_MC_knlGetCurProgramID
#define MC_knlGetParentProgramID	nEnv->E_MC_knlGetParentProgramID
#define MC_knlGetAppManagerID	nEnv->E_MC_knlGetAppManagerID
#define MC_knlGetProgramInfo	nEnv->E_MC_knlGetProgramInfo

#define MC_knlGetAccessLevel 	nEnv->E_MC_knlGetAccessLevel
#define MC_knlGetProgramName 	nEnv->E_MC_knlGetProgramName
#define MC_knlCurrentTime 		nEnv->E_MC_knlCurrentTime
#define MC_knlGetSystemProperty 	nEnv->E_MC_knlGetSystemProperty
#define MC_knlSetSystemProperty 	nEnv->E_MC_knlSetSystemProperty
#define MC_knlGetResourceID 	nEnv->E_MC_knlGetResourceID
#define MC_knlGetResource 		nEnv->E_MC_knlGetResource
#define MC_grpGetImageProperty 	nEnv->E_MC_grpGetImageProperty
#define MC_grpGetImageFrameBuffer 	nEnv->E_MC_grpGetImageFrameBuffer
#define MC_grpGetScreenFrameBuffer 	nEnv->E_MC_grpGetScreenFrameBuffer
#define MC_grpDestroyOffScreenFrameBuffer 	nEnv->E_MC_grpDestroyOffScreenFrameBuffer
#define MC_grpCreateOffScreenFrameBuffer 	nEnv->E_MC_grpCreateOffScreenFrameBuffer
#define MC_grpInitContext 		nEnv->E_MC_grpInitContext
#define MC_grpSetContext 		nEnv->E_MC_grpSetContext
#define MC_grpGetContext 		nEnv->E_MC_grpGetContext
#define MC_grpPutPixel 		nEnv->E_MC_grpPutPixel
#define MC_grpDrawLine 		nEnv->E_MC_grpDrawLine
#define MC_grpDrawRect 		nEnv->E_MC_grpDrawRect
#define MC_grpFillRect 		nEnv->E_MC_grpFillRect
#define MC_grpCopyFrameBuffer 	nEnv->E_MC_grpCopyFrameBuffer
#define MC_grpDrawImage 		nEnv->E_MC_grpDrawImage
#define MC_grpCopyArea 		nEnv->E_MC_grpCopyArea
#define MC_grpDrawArc 		nEnv->E_MC_grpDrawArc
#define MC_grpFillArc 		nEnv->E_MC_grpFillArc
#define MC_grpDrawString 		nEnv->E_MC_grpDrawString
#define MC_grpDrawUnicodeString 	nEnv->E_MC_grpDrawUnicodeString
#define MC_grpGetRGBPixels 		nEnv->E_MC_grpGetRGBPixels
#define MC_grpSetRGBPixels 		nEnv->E_MC_grpSetRGBPixels
#define MC_grpFlushLcd 		nEnv->E_MC_grpFlushLcd
#define MC_grpGetPixelFromRGB 	nEnv->E_MC_grpGetPixelFromRGB
#define MC_grpGetDisplayInfo 	nEnv->E_MC_grpGetDisplayInfo
#define MC_grpRepaint 		nEnv->E_MC_grpRepaint
#define MC_grpGetFont 		nEnv->E_MC_grpGetFont
#define MC_grpGetFontHeight 		nEnv->E_MC_grpGetFontHeight
#define MC_grpGetFontAscent 		nEnv->E_MC_grpGetFontAscent
#define MC_grpGetFontDescent 	nEnv->E_MC_grpGetFontDescent
#define MC_grpGetStringWidth 	nEnv->E_MC_grpGetStringWidth
#define MC_grpGetUnicodeStringWidth 	nEnv->E_MC_grpGetUnicodeStringWidth
#define MC_grpCreateImage 		nEnv->E_MC_grpCreateImage
#define MC_grpDestroyImage 		nEnv->E_MC_grpDestroyImage
#define MC_grpDecodeNextImage 	nEnv->E_MC_grpDecodeNextImage
#define MC_grpEncodeImage 		nEnv->E_MC_grpEncodeImage
#define MC_grpPostEvent 		nEnv->E_MC_grpPostEvent

#define MC_imGetSurpportModeCount nEnv->E_MC_imGetSurpportModeCount
#define MC_imGetSupportedModes	nEnv->E_MC_imGetSupportedModes
#define MC_imSetCurrentMode		nEnv->E_MC_imSetCurrentMode
#define MC_imGetCurrentMode		nEnv->E_MC_imGetCurrentMode
#define MC_imHandleInput		nEnv->E_MC_imHandleInput

#define 	MC_uicCreateApplicationContext 	nEnv->E_MC_uicCreateApplicationContext
#define 	MC_uicGetClass 			nEnv->E_MC_uicGetClass
#define 	MC_uicCreate 			nEnv->E_MC_uicCreate
#define 	MC_uicDestroy 			nEnv->E_MC_uicDestroy
#define 	MC_uicRepaint 			nEnv->E_MC_uicRepaint
#define 	MC_uicPaint 			nEnv->E_MC_uicPaint
#define 	MC_uicGetClassName 		nEnv->E_MC_uicGetClassName
#define 	MC_uicIsInstance 			nEnv->E_MC_uicIsInstance
#define 	MC_uicHandleEvent 		nEnv->E_MC_uicHandleEvent
#define 	MC_uicConfigure 			nEnv->E_MC_uicConfigure
#define 	MC_uicGetGeometry 		nEnv->E_MC_uicGetGeometry
#define 	MC_uicSetEnable 			nEnv->E_MC_uicSetEnable
#define 	MC_uicSetExtData 			nEnv->E_MC_uicSetExtData
#define 	MC_uicGetExtData 			nEnv->E_MC_uicGetExtData
#define 	MC_uicSetCallback 			nEnv->E_MC_uicSetCallback
#define 	MC_uicSetEventHandler 		nEnv->E_MC_uicSetEventHandler
#define 	MC_uicSetFont 			nEnv->E_MC_uicSetFont
#define 	MC_uicGetFont 			nEnv->E_MC_uicGetFont
#define 	MC_uicSetFgColor 			nEnv->E_MC_uicSetFgColor
#define 	MC_uicSetBgColor 			nEnv->E_MC_uicSetBgColor
#define 	MC_uicSetLabel 			nEnv->E_MC_uicSetLabel
#define 	MC_uicGetLabel 			nEnv->E_MC_uicGetLabel
#define 	MC_uicSetLabelAlignment 		nEnv->E_MC_uicSetLabelAlignment
#define 	MC_uicSetTimeMask 		nEnv->E_MC_uicSetTimeMask
#define 	MC_uicSetTime 			nEnv->E_MC_uicSetTime
#define 	MC_uicSetTimeLong 		nEnv->E_MC_uicSetTimeLong
#define 	MC_uicGetTime 			nEnv->E_MC_uicGetTime
#define 	MC_uicAddMenuItem 		nEnv->E_MC_uicAddMenuItem
#define 	MC_uicGetMenuItem 		nEnv->E_MC_uicGetMenuItem
#define 	MC_uicRemoveMenuItem 		nEnv->E_MC_uicRemoveMenuItem
#define 	MC_uicSetActiveMenuItem 		nEnv->E_MC_uicSetActiveMenuItem
#define 	MC_uicGetActiveMenuItem 		nEnv->E_MC_uicGetActiveMenuItem
#define 	MC_uicInsertText 			nEnv->E_MC_uicInsertText
#define 	MC_uicDeleteText 			nEnv->E_MC_uicDeleteText
#define 	MC_uicGetMaxTextSize 		nEnv->E_MC_uicGetMaxTextSize
#define 	MC_uicSetMaxTextSize 		nEnv->E_MC_uicSetMaxTextSize
#define 	MC_uicGetTextSize 			nEnv->E_MC_uicGetTextSize
#define 	MC_uicGetText 			nEnv->E_MC_uicGetText
#define 	MC_uicAddListItem 			nEnv->E_MC_uicAddListItem
#define 	MC_uicGetListItem 			nEnv->E_MC_uicGetListItem
#define 	MC_uicRemoveListItem 		nEnv->E_MC_uicRemoveListItem
#define 	MC_uicSetActiveListItem 		nEnv->E_MC_uicSetActiveListItem
#define 	MC_uicGetActiveListItem 		nEnv->E_MC_uicGetActiveListItem

// 네트웍 접근
#define MC_netConnect		nEnv->E_MC_netConnect
#define MC_netClose		nEnv->E_MC_netClose
// 네트웍 소켓
#define MC_netSocket		nEnv->E_MC_netSocket
#define MC_netSocketBind		nEnv->E_MC_netSocketBind
#define MC_netSocketConnect		nEnv->E_MC_netSocketConnect
#define MC_netSocketWrite		nEnv->E_MC_netSocketWrite
#define MC_netSocketRead		nEnv->E_MC_netSocketRead
#define MC_netSocketSendTo		nEnv->E_MC_netSocketSendTo
#define MC_netSocketRcvFrom	nEnv->E_MC_netSocketRcvFrom
#define MC_netSetWriteCB		nEnv->E_MC_netSetWriteCB
#define MC_netSetReadCB		nEnv->E_MC_netSetReadCB
#define MC_netSocketClose		nEnv->E_MC_netSocketClose
#define MC_netGetMaxPacketLength	nEnv->E_MC_netGetMaxPacketLength
#define MC_netGetHostAddr		nEnv->E_MC_netGetHostAddr
#define MC_netSocketAccept		nEnv->E_MC_netSocketAccept
// 네트웍 HTTP
#define MC_netHttpOpen		nEnv->E_MC_netHttpOpen
#define MC_netHttpConnect		nEnv->E_MC_netHttpConnect
#define MC_netHttpSetRequestMethod	nEnv->E_MC_netHttpSetRequestMethod
#define MC_netHttpGetRequestMethod	nEnv->E_MC_netHttpGetRequestMethod
#define MC_netHttpSetRequestProperty	nEnv->E_MC_netHttpSetRequestProperty
#define MC_netHttpGetRequestProperty	nEnv->E_MC_netHttpGetRequestProperty
#define MC_netHttpSetProxy		nEnv->E_MC_netHttpSetProxy
#define MC_netHttpGetProxy		nEnv->E_MC_netHttpGetProxy
#define MC_netHttpGetResponseCode	nEnv->E_MC_netHttpGetResponseCode
#define MC_netHttpGetResponseMessage	nEnv->E_MC_netHttpGetResponseMessage
#define MC_netHttpGetHeaderField	nEnv->E_MC_netHttpGetHeaderField
#define MC_netHttpGetLength		nEnv->E_MC_netHttpGetLength
#define MC_netHttpGetType		nEnv->E_MC_netHttpGetType
#define MC_netHttpGetEncoding	nEnv->E_MC_netHttpGetEncoding
#define MC_netHttpClose		nEnv->E_MC_netHttpClose
#define MC_netHandleEvent		nEnv->E_MC_netHandleEvent
// 시리얼
#define MC_srlOpen		nEnv->E_MC_srlOpen
#define MC_srlWrite		nEnv->E_MC_srlWrite
#define MC_srlRead		nEnv->E_MC_srlRead
#define MC_srlSetReadCB	nEnv->E_MC_srlSetReadCB
#define MC_srlSetWriteCB	nEnv->E_MC_srlSetWriteCB
#define MC_srlClose		nEnv->E_MC_srlClose
#define MC_srlHandleEvent	nEnv->E_MC_srlHandleEvent
//파일
#define MC_fsOpen		nEnv->E_MC_fsOpen
#define MC_fsRead		nEnv->E_MC_fsRead
#define MC_fsWrite		nEnv->E_MC_fsWrite
#define MC_fsClose		nEnv->E_MC_fsClose
#define MC_fsSeek		nEnv->E_MC_fsSeek
#define MC_fsFileAttribute	nEnv->E_MC_fsFileAttribute
#define MC_fsRemove	nEnv->E_MC_fsRemove
#define MC_fsRename	nEnv->E_MC_fsRename
#define MC_fsMkDir	nEnv->E_MC_fsMkDir
#define MC_fsRmDir	nEnv->E_MC_fsRmDir
#define MC_fsList		nEnv->E_MC_fsList
#define MC_fsTotalSpace	nEnv->E_MC_fsTotalSpace
#define MC_fsAvailable	nEnv->E_MC_fsAvailable
//MDA
#define MC_mdaClipCreate	nEnv->E_MC_mdaClipCreate
#define MC_mdaClipFree	nEnv->E_MC_mdaClipFree
#define MC_mdaSetWaterMark	nEnv->E_MC_mdaSetWaterMark
#define MC_mdaClipGetType	nEnv->E_MC_mdaClipGetType
#define MC_mdaClipPutData	nEnv->E_MC_mdaClipPutData
#define MC_mdaClipPutToneData	nEnv->E_MC_mdaClipPutToneData
#define MC_mdaClipPutFreqToneData	nEnv->E_MC_mdaClipPutFreqToneData
#define MC_mdaClipGetData	nEnv->E_MC_mdaClipGetData
#define MC_mdaClipAvailableDataSize 	nEnv->E_MC_mdaClipAvailableDataSize
#define MC_mdaClipClearData		nEnv->E_MC_mdaClipClearData
#define MC_mdaPlay		nEnv->E_MC_mdaPlay
#define MC_mdaPause		nEnv->E_MC_mdaPause
#define MC_mdaResume		nEnv->E_MC_mdaResume
#define MC_mdaStop		nEnv->E_MC_mdaStop
#define MC_mdaRecord		nEnv->E_MC_mdaRecord
#define MC_mdaGetVolume		nEnv->E_MC_mdaGetVolume
#define MC_mdaSetVolume		nEnv->E_MC_mdaSetVolume
#define MC_mdaVibrator		nEnv->E_MC_mdaVibrator

// DB
#define MC_dbOpenDataBase       nEnv->E_MC_dbOpenDataBase
#define MC_dbCloseDataBase      nEnv->E_MC_dbCloseDataBase 
#define MC_dbDeleteDataBase     nEnv->E_MC_dbDeleteDataBase 
#define MC_dbInsertRecord       nEnv->E_MC_dbInsertRecord
#define MC_dbSelectRecord       nEnv->E_MC_dbSelectRecord
#define MC_dbUpdateRecord       nEnv->E_MC_dbUpdateRecord
#define MC_dbDeleteRecord       nEnv->E_MC_dbDeleteRecord
#define MC_dbListRecords        nEnv->E_MC_dbListRecords
#define MC_dbSortRecords        nEnv->E_MC_dbSortRecords
#define MC_dbGetAccessMode      nEnv->E_MC_dbGetAccessMode
#define MC_dbGetNumberOfRecords nEnv->E_MC_dbGetNumberOfRecords
#define MC_dbGetRecordSize      nEnv->E_MC_dbGetRecordSize
#define MC_dbListDataBases      nEnv->E_MC_dbListDataBases

// util
#define MC_utilGetResource	nEnv->E_MC_utilGetResource
//#define MC_utilGetDate nEnv->E_MC_utilGetDate
//#define MC_utilComputeDate nEnv->E_MC_utilComputeDate
#define MC_utilHtonl nEnv->E_MC_utilHtonl
#define MC_utilHtons nEnv->E_MC_utilHtons
#define MC_utilNtohl nEnv->E_MC_utilNtohl
#define MC_utilNtohs nEnv->E_MC_utilNtohs
#define MC_utilInetAddrInt nEnv->E_MC_utilInetAddrInt
#define MC_utilInetAddrStr nEnv->E_MC_utilInetAddrStr
//phone
#define MC_phnCallPlace nEnv->E_MC_phnCallPlace
#define MC_phnSmsOpen nEnv->E_MC_phnSmsOpen
#define MC_phnGetSMSAvailable nEnv->E_MC_phnGetSMSAvailable
#define MC_phnSmsClose nEnv->E_MC_phnSmsClose
#define MC_phnSmsRead nEnv->E_MC_phnSmsRead
#define MC_phnSmsSend nEnv->E_MC_phnSmsSend


// misc
#define MC_miscBackLight nEnv->E_MC_miscBackLight
#define MC_miscSetLed nEnv->E_MC_miscSetLed
#define MC_miscGetLed nEnv->E_MC_miscGetLed
#define MC_miscGetLedCount nEnv->E_MC_miscGetLedCount

#define MC_knlDestroySharedBuf nEnv->E_MC_knlDestroySharedBuf



#define MN_FBEGIN(rtnType, func)    __declspec(dllexport)  void   func(
#define MN_END                      }

#define MN_WORD(type, p)		type p = *(type *)((*(nEnv->Java_sp))++)
#define MN_DWORD_HI(type, p)	jint dummy_##p = *(jint *)((*(nEnv->Java_sp))++)
#define MN_DWORD_LO(type, p)	type p = *(type *)((*(nEnv->Java_sp))++)

#define MN_DWORD(type, p)       type p = *(type *)((*(nEnv->Java_sp))++); \
                                jint dummy_##p = *(jint *)((*(nEnv->Java_sp))++)


#define MN_NO_PARM()	MG_NativeEnv* nEnv) {

#define MN_PARM1(type0)            MG_NativeEnv* nEnv ) { \
		type0;

#define MN_PARM2(type0, type1)     MG_NativeEnv* nEnv  ) { \
		type1; \
		type0;

#define MN_PARM3(type0, type1, type2)       MG_NativeEnv* nEnv) { \
		type2; \
		type1; \
		type0;

#define MN_PARM4(type0, type1, type2, type3)        MG_NativeEnv* nEnv) { \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM5(type0, type1, type2, type3, type4)     MG_NativeEnv* nEnv) { \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM6(type0, type1, type2, type3, type4, type5)    MG_NativeEnv* nEnv ) { \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM7(type0, type1, type2, type3, type4, type5, type6)    MG_NativeEnv* nEnv ) { \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM8(type0, type1, type2, type3, type4, type5, type6, type7)     MG_NativeEnv* nEnv) { \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM9(type0, type1, type2, type3, type4, type5, type6, type7, type8)     MG_NativeEnv* nEnv) { \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM10(type0, type1, type2, type3, type4, type5, type6, type7, type8, type9)    MG_NativeEnv* nEnv ) { \
		type9; \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM11(type0, type1, type2, type3, type4, type5, type6, type7, type8, type9, type10)    MG_NativeEnv* nEnv ) { \
		type10; \
		type9; \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_PARM12(type0, type1, type2, type3, type4, type5, type6, type7, type8, type9, type10, type11)     MG_NativeEnv* nEnv) { \
		type11; \
		type10; \
		type9; \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM1(type0)            MG_NativeEnv* nEnv ) { \
		type0;

#define MN_ST_PARM2(type0, type1)     MG_NativeEnv* nEnv  ) { \
		type1; \
		type0;

#define MN_ST_PARM3(type0, type1, type2)       MG_NativeEnv* nEnv) { \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM4(type0, type1, type2, type3)        MG_NativeEnv* nEnv) { \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM5(type0, type1, type2, type3, type4)     MG_NativeEnv* nEnv) { \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM6(type0, type1, type2, type3, type4, type5)    MG_NativeEnv* nEnv ) { \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM7(type0, type1, type2, type3, type4, type5, type6)    MG_NativeEnv* nEnv ) { \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM8(type0, type1, type2, type3, type4, type5, type6, type7)     MG_NativeEnv* nEnv) { \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM9(type0, type1, type2, type3, type4, type5, type6, type7, type8)     MG_NativeEnv* nEnv) { \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM10(type0, type1, type2, type3, type4, type5, type6, type7, type8, type9)    MG_NativeEnv* nEnv ) { \
		type9; \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM11(type0, type1, type2, type3, type4, type5, type6, type7, type8, type9, type10)    MG_NativeEnv* nEnv ) { \
		type10; \
		type9; \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;

#define MN_ST_PARM12(type0, type1, type2, type3, type4, type5, type6, type7, type8, type9, type10, type11)     MG_NativeEnv* nEnv) { \
		type11; \
		type10; \
		type9; \
		type8; \
		type7; \
		type6; \
		type5; \
		type4; \
		type3; \
		type2; \
		type1; \
		type0;
		
#define MN_RETVAL(rtn)         *(--(int*)(*(nEnv->Java_sp))) = (jint)(rtn)
#define MN_RETVAL_FLOAT(rtn)         *(--(float*)(*(nEnv->Java_sp))) = (float)(rtn)
#define MN_RETVAL_LONG(rtn)    *(nEnv->Java_sp) -= 2; \
				*((jlong*)(void*)(*(nEnv->Java_sp))) = rtn
#define MN_RETVAL_DOUBLE(rtn)    *(nEnv->Java_sp) -= 2; \
				*((jdouble*)(void*)(*(nEnv->Java_sp))) = rtn
	
extern jobject* newArrayTypeClasses;
			
#define TYPE_Boolean    4
#define TYPE_Char       5
#define TYPE_Float      6
#define TYPE_Double     7
#define TYPE_Byte       8
#define TYPE_Short      9
#define TYPE_Int        10
#define TYPE_Long       11
#define TYPE_Ref        15


/**
	MH_FB_MAIN_LCD
	<br>#define MH_FB_MAIN_LCD		1
*/	
#define MH_FB_MAIN_LCD		1
/**
	MH_FB_SUB_LCD
	<br>#define MH_FB_SUB_LCD		2
*/
#define MH_FB_SUB_LCD		2
