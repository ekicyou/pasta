using System;
using System.Collections.Generic;

namespace NeoDatis.Tool.Wrappers.Map
{
   [System.Serializable]
   public class OdbHashMap<TKey, TValue> : System.Collections.Generic.IDictionary<TKey, TValue>
   {
      public OdbHashMap()
      : base()
      {
          dictionary = new Dictionary<TKey, TValue>();
      }
      
      protected IDictionary<TKey,TValue> dictionary;

      public OdbHashMap(int capacity)
      {
         dictionary = new Dictionary<TKey,TValue>(capacity);
      }
      
      public OdbHashMap(IDictionary<TKey, TValue> dic)
      {
         dictionary = new Dictionary<TKey,TValue>();
         PutAll(dic);
      }
      
      public virtual bool PutAll(IDictionary<TKey, TValue> map)
      {
         
         ICollection<TKey> keys = map.Keys;
         foreach(TKey k in keys){
            Add(k,map[k]);
         }
         return true;
      }
      
      public virtual bool RemoveAll(IDictionary<TKey, TValue> map)
      {
         ICollection<TKey> keys = map.Keys;
         foreach (TKey k in keys)
         {
            Remove(k);
         }
         return true;
      }
      
      public virtual void Put(TKey k, TValue v)
      {
         Add(k, v); ;
      }
      public TValue Get(TKey k)
      {
         TValue v = default(TValue);
         this.TryGetValue(k, out v);
         return v;
      }
      public TValue Remove2(TKey key)
      {
         
         TValue v = default(TValue);
         TryGetValue(key,out v);
         if(v!=null){
            Remove(key);
         }
         return v;
      }
      // allow array-like access
      public TValue this[TKey key]
      {
         get
         {
            if(key==null)
             {
               return default(TValue);
            }
            TValue v = default(TValue);
            this.TryGetValue(key,out v);
            return v;
         }
         set
         {
            if(key==null)
             {
               return;
            }
            dictionary[key] = value;
         }
      }
      public void Add(TKey key,TValue v)
      {
          TValue vnull = default(TValue);
          this.TryGetValue(key, out vnull);
          if (vnull != null)
          {
              Remove(key);
          }
          dictionary.Add(key,v);
      }
      public void Add(KeyValuePair<TKey,TValue> item)
      {
         dictionary.Add(item);
      }
      public void Clear()
      {
         dictionary.Clear();
      }
      public bool ContainsKey(TKey key)
      {
         return dictionary.ContainsKey( key);
      }
      
      public bool Contains(KeyValuePair<TKey,TValue> item)
      {
         return dictionary.Contains(item);
      }
      
      public void CopyTo(KeyValuePair<TKey,TValue> [] array,int arrayIndex)
      {
         dictionary.CopyTo(array,arrayIndex);
      }
      public IEnumerator<KeyValuePair<TKey,TValue>> GetEnumerator()
      {
         return dictionary.GetEnumerator();
      }
      public bool TryGetValue(TKey key,out TValue v)
      {
         return dictionary.TryGetValue(key,out v);
      }
      
      public bool Remove(KeyValuePair<TKey,TValue> item)
      {
         return dictionary.Remove(item);
      }
      
      public bool Remove(TKey key)
      {
         return dictionary.Remove(key);
      }
      public virtual int Count
      {
         get
         {
            return dictionary.Count;
         }
      }
      
      public virtual bool IsReadOnly
      {
         get
         {
            return dictionary.IsReadOnly;
         }
      }
      public ICollection<TValue> Values 
      { 
         get
         {
            return dictionary.Values;
         }
         
      }
      public ICollection<TKey> Keys 
      { 
         get
         {
            return dictionary.Keys;
         }
         
      }
      #region IEnumerable Members

      System.Collections.IEnumerator System.Collections.IEnumerable.GetEnumerator()
      {
          return this.dictionary as System.Collections.IEnumerator;

      }

      #endregion
   }
   
}
