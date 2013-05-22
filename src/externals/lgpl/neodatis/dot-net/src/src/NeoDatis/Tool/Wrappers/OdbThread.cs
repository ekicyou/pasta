
using System;
using System.Threading;

namespace NeoDatis.Tool.Wrappers
{
   [Serializable]
   public class OdbThread {
      public OdbThread(): base() {
         
      }
      
      public OdbThread(OdbRunnable target) {
         
      }
      
      
      public void Start(){
      }
      
      public static String GetCurrentThreadName(){
         return  Thread.CurrentThread.Name;
      }
      public void Interrupt(){
         
      }	
      
      public static void Sleep(long timeout)
      {
         int t = (int) timeout;
         Thread.Sleep(t);
      }
      public string GetName(){
         return "no-name";
      }
      
   }
}
