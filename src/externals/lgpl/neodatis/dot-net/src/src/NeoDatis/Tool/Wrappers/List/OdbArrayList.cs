namespace NeoDatis.Tool.Wrappers.List
{
	[System.Serializable]
	public class OdbArrayList<E> : System.Collections.Generic.List<E>, IOdbList<E>
	{
		public OdbArrayList() : base(){
		}

		public OdbArrayList(int size) : base(size){
		}
	
		public virtual bool AddAll(System.Collections.Generic.ICollection<E> c)
		{
			base.AddRange(c);
			return true;
		}

		public virtual bool RemoveAll(System.Collections.Generic.ICollection<E> c)
		{
            foreach (E e in c)
            {
                Remove(e);
            }
            return true;
		}
	
		public virtual E Get(int index)
		{
			return base[index];
		}
		public virtual bool IsEmpty()
		{
			return Count==0;
		}
		public void Set(int index, E element)
		{
			Insert(index,element);
		}
		
	}
}
