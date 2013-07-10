//******************************************************************************
// short ComputeCrc16 -- Compute 16 bit CRC
//
// This static method calculates the the 16 bit CRC.  The broadcast
// packets use a 16 bit cyclic redundancy checksum (CRC) using a
// polynomial of the form X5 + X. The packet excluding trailing
// CRC-code must be processed through this routine.
//
//******************************************************************************
#define POLY_HIBIT (1L<<16)
#define POLY_MASK (POLY_HIBIT-1)
#define POLY_NOME (POLY_HIBIT+(1<<5)+1)
short ComputeCrc16( const void * pktIP, // Ptr to the packet start
                    int lenI            // Len to be processed thru char preceeding CRC
                  )
{
    int crc = 0, val, byteX, bitX;
    const unsigned char * pktP = (const unsigned char *) pktIP;

    for ( byteX=0; byteX < lenI; byteX++ )
    {

        val = pktP[byteX];

        for (bitX=0; bitX<8; bitX++)
        {
            crc = (crc<<1) | (val & 1);
            val >>= 1;

            if (crc & POLY_HIBIT) {
                crc ^= POLY_NOME;
            }
        }
    }
    return (short)(crc & POLY_MASK);

}
/*

上の処理を８bit単位に変換すると‥‥

計算前のCRC [CH][CL]（上位8bit / 下位8bit）
入力値  B

換算：[CRCの下位８bit][入力値のビット列を逆転させた値]

換算値に、[CRC上位8bitから求めたbit反転列]をXOR

→bit反転の配列と、XOR配列をそれぞれ作成して利用

*/
