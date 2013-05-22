using System.Collections;
using System.Collections.Generic;

namespace NeoDatis.Tool.Wrappers.List
{
   public class NeoDatisCollectionUtil
   {
      
      public static IList append(IList c1, IList c2)
      {
         IEnumerator enumerator = c2.GetEnumerator();
         
         while(enumerator.MoveNext())
         {
            object o = enumerator.Current;
            c1.Add(o);
         }
         
         return c1;
      }
      
      public static System.Collections.IList Sublist(System.Collections.IList l1, int from, int to)
      {
         System.Collections.IList l = new ArrayList();
         for(int i=from;i<to;i++)
         {
            l.Add(l1[i]);	
         }
         return l;
      }
      
       public static System.Collections.Generic.IList<object> SublistGeneric(System.Collections.Generic.IList<object> l1, int from, int to)
      {
         System.Collections.Generic.List<object> l = new List<object>();
         for(int i=from;i<to;i++)
         {
            l.Add(l1[i]);	
         }
         return l;
      }
      
   }
}
