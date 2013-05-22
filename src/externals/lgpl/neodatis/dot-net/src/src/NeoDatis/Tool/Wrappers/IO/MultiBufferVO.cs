using System;
using System.Collections.Generic;
using System.Text;
using System.IO;

namespace NeoDatis.Tool.Wrappers.IO{
   public class MultiBufferVO
   {
      ///<summary> The number of buffers///</summary>
      private int numberOfBuffers;
      ///<summary> The buffer size///</summary>
      private int bufferSize;
      public byte [][] buffers;
      
      protected long[] creations;
      ///<summary> The current start position of the buffer ///</summary>
      public long[] bufferStartPosition;
      
      ///<summary> The current end position of the buffer ///</summary>
	public long[] bufferEndPosition;
      
      ///<summary>
      /// The max position in the buffer, used to optimize the flush - to flush
      /// only new data and not all the buffer
      ///</summary>
      public int[] maxPositionInBuffer;
      
      ///<summary> To know if buffer has been used for write - to speedup flush ///</summary>
      private bool[] bufferHasBeenUsedForWrite;
      
      public MultiBufferVO(int numberOfBuffers, int bufferSize){
         this.numberOfBuffers = numberOfBuffers;
         this.bufferSize = bufferSize;
         buffers = new byte[numberOfBuffers][];
         
         for (int x = 0; x < numberOfBuffers; x++) 
         {
            buffers[x] = new byte[bufferSize];
         }
         
         bufferStartPosition = new long[numberOfBuffers];
         bufferEndPosition = new long[numberOfBuffers];
         maxPositionInBuffer = new int[numberOfBuffers];
         creations = new long[numberOfBuffers];
         bufferHasBeenUsedForWrite = new bool[numberOfBuffers];
	}
      
      public byte[] GetBuffer2(int index){
         return buffers[index];
      }
      public byte GetByte(int bufferIndex, int byteIndex){
         return buffers[bufferIndex][byteIndex];
      }
      
      ///<summary>
      /// @param i
      ///</summary>
      public void ClearBuffer(int bufferIndex) {
         byte [] buffer = buffers[bufferIndex];
         int maxPosition = maxPositionInBuffer[bufferIndex];
         for (int i = 0; i < maxPosition; i++) {
            buffer[i] = 0;
         }
         bufferStartPosition[bufferIndex]=0;
         bufferEndPosition[bufferIndex]=0;
         maxPositionInBuffer[bufferIndex]=0;
         bufferHasBeenUsedForWrite[bufferIndex]=false;
      }
      
      ///<summary>
      /// @param bufferIndex
      /// @param positionInBuffer
      /// @param b
      ///</summary>
      public void SetByte(int bufferIndex, int positionInBuffer, byte b) {
         if(buffers[bufferIndex]==null){
            buffers[bufferIndex] = new byte[bufferSize];
         }
         buffers[bufferIndex][positionInBuffer] = b;
         bufferHasBeenUsedForWrite[bufferIndex] = true;
         if (positionInBuffer > maxPositionInBuffer[bufferIndex]) {
            maxPositionInBuffer[bufferIndex] = positionInBuffer;
         }
      }
      
      public int GetBufferIndexForPosition(long position, int size) {
		long max = position + size;
         
		for (int i = 0; i < numberOfBuffers; i++) {
            // Check if new position is in buffer
            if (max <= bufferEndPosition[i] && position >= bufferStartPosition[i]) {
               return i;
            }
         }
         return -1;
      }
      
      ///<summary>
      /// @param bufferIndex
      /// @param currentTimeInMs
      ///</summary>
      public void SetCreationDate(int bufferIndex, long currentTimeInMs) {
         creations[bufferIndex] = currentTimeInMs;
         
      }
      
      ///<summary>
      /// @param bufferIndex
      /// @param newPosition
      /// @param endPosition
      /// @param i
      ///</summary>
      public void SetPositions(int bufferIndex, long startPosition, long endPosition, int maxPosition) {
         bufferStartPosition[bufferIndex] = startPosition;
         bufferEndPosition[bufferIndex] = endPosition;
		maxPositionInBuffer[bufferIndex] = maxPosition;
      }
      
      
      private void Clear(int bufferIndex, int position) {
         ////
         /// if (buffer == null) { buffer = new byte[bufferSize]; return; }
         ///</summary>
         byte[] buffer = buffers[bufferIndex];
         for (int i = 0; i < position; i++) {
            buffer[i] = 0;
         }
         bufferStartPosition[bufferIndex] = 0;
         bufferEndPosition[bufferIndex] = 0;
         maxPositionInBuffer[bufferIndex] = 0;
         bufferHasBeenUsedForWrite[bufferIndex] = false;
         
      }
      
      ///<summary>
      /// @param bufferIndex
      /// @param bytes
      /// @param startIndex
      /// @param i
      /// @param lengthToCopy
      ///</summary>
      public void WriteBytes(int bufferIndex, byte[] bytes, int startIndex, int offsetWhereToCopy, int lengthToCopy) {
		Array.Copy(bytes, startIndex, buffers[bufferIndex], offsetWhereToCopy, lengthToCopy);
         
         bufferHasBeenUsedForWrite[bufferIndex] = true;
         
         int positionInBuffer = offsetWhereToCopy + lengthToCopy - 1;
         if (positionInBuffer > maxPositionInBuffer[bufferIndex]) {
            maxPositionInBuffer[bufferIndex] = positionInBuffer;
         }
         
      }
      
      ///<summary>
      /// @param bufferIndex
      /// @return
      ///</summary>
      public bool HasBeenUsedForWrite(int bufferIndex) {
         return bufferHasBeenUsedForWrite[bufferIndex];
      }
      
      ///<summary>
      /// 
      ///</summary>
      public void Clear() {
         
         buffers = null;
         bufferStartPosition = null;
         bufferEndPosition = null;
         maxPositionInBuffer = null;
         bufferHasBeenUsedForWrite = null;
      }
      
      ///<summary>
      /// @param i
      /// @return
      ///</summary>
      public long GetCreationDate(int bufferIndex) {
         return creations[bufferIndex];
      }
      
   }
   
}