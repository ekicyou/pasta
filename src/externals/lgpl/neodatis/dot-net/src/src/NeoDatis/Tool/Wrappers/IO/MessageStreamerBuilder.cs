using NeoDatis.Odb.Core.Server.Layers.Layer3.Engine;
using System.Net.Sockets;

/// @port.todo
namespace NeoDatis.Tool.Wrappers.IO{
   public class MessageStreamerBuilder
   {
      public static IMessageStreamer GetMessageStreamer(TcpClient socket) {
         return null;		
      }
      /**
      * 
      */
      public static IMessageStreamer GetMessageStreamer(string host, int port, string name) {
         return null;
      }
   }
}