namespace NeoDatis.Tool.Mutex
{
	/// <summary>A Simple Mutex for lock operations</summary>
	/// <author>osmadja</author>
	public class Mutex
	{
		/// <summary>The name of the mutex</summary>
		private string name;

		/// <summary>The lock status *</summary>
		protected bool inUse;

		protected int nbOwners;

		private bool debug;

		public Mutex(string name)
		{
			this.name = name;
			this.inUse = false;
			this.debug = false;
			this.nbOwners = 0;
		}

		/// <exception cref="System.Exception"></exception>
		public virtual NeoDatis.Tool.Mutex.Mutex Acquire(string who)
		{
			if (debug)
			{
				NeoDatis.Tool.DLogger.Info("Thread " + NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
					() + " - " + who + " : Trying to acquire mutex " + name);
			}
			//DLogger.info("From " + StringUtils.exceptionToString(new Exception(), false));
			//if (Java.Lang.Thread.Interrupted())
			//{
			//	throw new System.Exception();
			//}
			lock (this)
			{
				try
				{
					while (inUse)
					{
						//Sharpen.Runtime.Wait(this);
					}
					if (nbOwners != 0)
					{
						throw new System.Exception("nb owners != 0 - " + nbOwners);
					}
					inUse = true;
					nbOwners++;
				}
				catch (System.Exception ex)
				{
					//Sharpen.Runtime.Notify(this);
					throw;
				}
			}
			if (debug)
			{
				NeoDatis.Tool.DLogger.Info("Thread " + NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
					() + " - " + who + " : Mutex " + name + " acquired!");
			}
			return this;
		}

		public virtual void Release(string who)
		{
			lock (this)
			{
				if (debug)
				{
					NeoDatis.Tool.DLogger.Info("Thread " + NeoDatis.Tool.Wrappers.OdbThread.GetCurrentThreadName
						() + " - " + who + " : Releasing mutex " + name);
				}
				inUse = false;
				nbOwners--;
				if (nbOwners < 0)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.InternalError
						.AddParameter("Nb owner is negative in release(" + who + ")"));
				}
				//Sharpen.Runtime.Notify(this);
			}
		}

		/// <exception cref="System.Exception"></exception>
		public virtual bool Attempt(long msecs)
		{
			//if (Java.Lang.Thread.Interrupted())
			//{
				//throw new System.Exception();
			//}
			lock (this)
			{
				if (!inUse)
				{
					inUse = true;
					nbOwners++;
					return true;
				}
				else
				{
					if (msecs <= 0)
					{
						return false;
					}
					else
					{
						long waitTime = msecs;
						long start = NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs();
						try
						{
							for (; ; )
							{
								//Sharpen.Runtime.Wait(this, waitTime);
								if (!inUse)
								{
									inUse = true;
									nbOwners++;
									return true;
								}
								waitTime = msecs - (NeoDatis.Tool.Wrappers.OdbTime.GetCurrentTimeInMs() - start);
								if (waitTime <= 0)
								{
									return false;
								}
							}
						}
						catch (System.Exception ex)
						{
							//Sharpen.Runtime.Notify(this);
							throw;
						}
					}
				}
			}
		}

		public virtual string GetName()
		{
			return name;
		}

		public virtual void SetDebug(bool debug)
		{
			this.debug = debug;
		}

		public virtual bool IsInUse()
		{
			return inUse;
		}

		public virtual int GetNbOwners()
		{
			return nbOwners;
		}
	}
}
