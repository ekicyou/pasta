namespace NeoDatis.Odb
{
	/// <summary>The main NeoDatis ODB Configuration class.</summary>
	/// <remarks>
	/// The main NeoDatis ODB Configuration class. All engine configuration is done
	/// via this class.
	/// </remarks>
	/// <author>osmadja</author>
	public class OdbConfiguration
	{
		private static bool coreProviderInit = false;

		private static bool debugEnabled = false;

		private static bool logAll = false;

		private static int debugLevel = 100;

		private static System.Collections.Generic.IDictionary<string, string> logIds = null;

		private static bool infoEnabled = false;

		private static bool enableAfterWriteChecking = false;

		private static int maxNumberOfWriteObjectPerTransaction = 10000;

		private static long maxNumberOfObjectInCache = 3000000;

		private static int defaultBufferSizeForData = 1024 * 16;

		private static int defaultBufferSizeForTransaction = 4096 * 4;

		private static int nbBuffers = 5;

		private static bool useMultiBuffer = true;

		private static bool automaticCloseFileOnExit = false;

		private static bool saveHistory = false;

		private static string defaultDatabaseCharacterEncoding = "ISO8859-1";

		private static string databaseCharacterEncoding = defaultDatabaseCharacterEncoding;

		private static bool throwExceptionWhenInconsistencyFound = true;

		private const int NbIdsPerBlock = 1000;

		private const int IdBlockRepetitionSize = 18;

		/// <summary>header(34) + 1000 * 18</summary>
		private static int idBlockSize = 34 + NbIdsPerBlock * IdBlockRepetitionSize;

		private static bool inPlaceUpdate = false;

		private static int stringSpaceReserveFactor = 1;

		private static bool checkModelCompatibility = true;

		private static bool monitorMemory = false;

		/// <summary>
		/// A boolean value to indicate if ODB can create empty constructor when not
		/// available
		/// </summary>
		private static bool enableEmptyConstructorCreation = true;

		/// <summary>
		/// a boolean value to specify if ODBFactory waits a little to re-open a file
		/// when a file is locked
		/// </summary>
		private static bool retryIfFileIsLocked = true;

		/// <summary>How many times ODBFactory tries to open the file when it is locked</summary>
		private static int numberOfRetryToOpenFile = 5;

		/// <summary>How much time (in ms) ODBFactory waits between each retry</summary>
		private static long retryTimeout = 100;

		/// <summary>How much time (in ms) ODBFactory waits to be sure a file has been created
		/// 	</summary>
		private static long defaultFileCreationTime = 500;

		/// <summary>Automatically increase cache size when it is full</summary>
		private static bool automaticallyIncreaseCacheSize = false;

		private static bool useCache = true;

		private static bool logServerStartupAndShutdown = true;

		private static bool logServerConnections = false;

		/// <summary>The default btree size for index btrees</summary>
		private static int defaultIndexBTreeDegree = 20;

		/// <summary>The type of cache.</summary>
		/// <remarks>
		/// The type of cache. If true, the cache use weak references that allows
		/// very big inserts,selects like a million of objects. But it is a little
		/// bit slower than setting to false
		/// </remarks>
		private static bool useLazyCache = false;

		/// <summary>To indicate if warning must be displayed</summary>
		private static bool displayWarnings = true;

		private static NeoDatis.Odb.Core.Query.Execution.IQueryExecutorCallback queryExecutorCallback
			 = null;

		/// <summary>Scale used for average action *</summary>
		private static int scaleForAverageDivision = 2;

		/// <summary>Round Type used for the average division</summary>
		private static int roundTypeForAverageDivision = NeoDatis.Tool.Wrappers.ConstantWrapper
			.RoundTypeForAverageDivision;

		/// <summary>for IO atomic writing&reading</summary>
		private static System.Type ioClass = typeof(NeoDatis.Tool.Wrappers.IO.OdbFileIO);

		/// <summary>for IO atomic : password for encryption</summary>
		private static string encryptionPassword;

		/// <summary>The core provider is the provider of core object implementation for ODB</summary>
		private static NeoDatis.Odb.Core.ICoreProvider coreProvider = new NeoDatis.Odb.Impl.DefaultCoreProvider
			();

		/// <summary>To indicate if NeoDatis must check the runtime version, defaults to yes</summary>
		private static bool checkRuntimeVersion = true;

		/// <summary>
		/// To specify if NeoDatis must automatically reconnect objects loaded in previous
		/// session.
		/// </summary>
		/// <remarks>
		/// To specify if NeoDatis must automatically reconnect objects loaded in previous
		/// session. With with flag on, user does not need to manually reconnect an
		/// object. Default value = true
		/// </remarks>
		private static bool reconnectObjectsToSession = false;

		private static NeoDatis.Tool.Wrappers.ClassLoader classLoader = NeoDatis.Tool.Wrappers.NeoDatisClassLoader
			.GetCurrent();

		private static System.Type messageStreamerClass = typeof(NeoDatis.Odb.Impl.Core.Server.Layers.Layer3.Engine.DefaultMessageStreamer
			);

		/// <summary>To activate or desactivate the use of index</summary>
		private static bool useIndex = true;

		// For multi thread
		/// <returns></returns>
		public static bool ReconnectObjectsToSession()
		{
			return reconnectObjectsToSession;
		}

		public static void SetReconnectObjectsToSession(bool reconnectObjectsToSession)
		{
			NeoDatis.Odb.OdbConfiguration.reconnectObjectsToSession = reconnectObjectsToSession;
		}

		public static int GetDefaultBufferSizeForData()
		{
			return defaultBufferSizeForData;
		}

		public static void SetDefaultBufferSizeForData(int defaultBufferSize)
		{
			NeoDatis.Odb.OdbConfiguration.defaultBufferSizeForData = defaultBufferSize;
		}

		public static void AddLogId(string logId)
		{
			if (logIds == null)
			{
				logIds = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, string>();
			}
			logIds.Add(logId, logId);
		}

		public static void RemoveLogId(string logId)
		{
			if (logIds == null)
			{
				logIds = new NeoDatis.Tool.Wrappers.Map.OdbHashMap<string, string>();
			}
			logIds.Remove(logId);
		}

		public static bool IsDebugEnabled(string logId)
		{
			if (!debugEnabled)
			{
				return false;
			}
			if (logAll)
			{
				return true;
			}
			if (logIds == null || logIds.Count == 0)
			{
				return false;
			}
			return logIds.ContainsKey(logId);
		}

		public static void SetDebugEnabled(int level, bool debug)
		{
			NeoDatis.Odb.OdbConfiguration.debugEnabled = debug;
			NeoDatis.Odb.OdbConfiguration.debugLevel = level;
		}

		public static bool IsEnableAfterWriteChecking()
		{
			return enableAfterWriteChecking;
		}

		public static bool IsInfoEnabled()
		{
			return infoEnabled;
		}

		public static bool IsInfoEnabled(string logId)
		{
			// return false;
			if (logAll)
			{
				return true;
			}
			if (logIds == null || logIds.Count == 0)
			{
				return false;
			}
			return logIds.ContainsKey(logId);
		}

		// return false;
		public static void SetInfoEnabled(bool infoEnabled)
		{
			NeoDatis.Odb.OdbConfiguration.infoEnabled = infoEnabled;
		}

		public static void SetEnableAfterWriteChecking(bool enableAfterWriteChecking)
		{
			NeoDatis.Odb.OdbConfiguration.enableAfterWriteChecking = enableAfterWriteChecking;
		}

		public static int GetMaxNumberOfWriteObjectPerTransaction()
		{
			return maxNumberOfWriteObjectPerTransaction;
		}

		public static void SetMaxNumberOfWriteObjectPerTransaction(int maxNumberOfWriteObjectPerTransaction
			)
		{
			NeoDatis.Odb.OdbConfiguration.maxNumberOfWriteObjectPerTransaction = maxNumberOfWriteObjectPerTransaction;
		}

		public static long GetMaxNumberOfObjectInCache()
		{
			return maxNumberOfObjectInCache;
		}

		public static void SetMaxNumberOfObjectInCache(long maxNumberOfObjectInCache)
		{
			NeoDatis.Odb.OdbConfiguration.maxNumberOfObjectInCache = maxNumberOfObjectInCache;
		}

		public static int GetNumberOfRetryToOpenFile()
		{
			return numberOfRetryToOpenFile;
		}

		public static void SetNumberOfRetryToOpenFile(int numberOfRetryToOpenFile)
		{
			NeoDatis.Odb.OdbConfiguration.numberOfRetryToOpenFile = numberOfRetryToOpenFile;
		}

		public static long GetRetryTimeout()
		{
			return retryTimeout;
		}

		public static void SetRetryTimeout(long retryTimeout)
		{
			NeoDatis.Odb.OdbConfiguration.retryTimeout = retryTimeout;
		}

		public static bool RetryIfFileIsLocked()
		{
			return retryIfFileIsLocked;
		}

		public static void SetRetryIfFileIsLocked(bool retryIfFileIsLocked)
		{
			NeoDatis.Odb.OdbConfiguration.retryIfFileIsLocked = retryIfFileIsLocked;
		}

		public static long GetDefaultFileCreationTime()
		{
			return defaultFileCreationTime;
		}

		public static void SetDefaultFileCreationTime(long defaultFileCreationTime)
		{
			NeoDatis.Odb.OdbConfiguration.defaultFileCreationTime = defaultFileCreationTime;
		}

		public static bool IsMultiThread()
		{
			return retryIfFileIsLocked;
		}

		public static void UseMultiThread(bool yes)
		{
			UseMultiThread(yes, numberOfRetryToOpenFile);
		}

		public static void UseMultiThread(bool yes, int numberOfThreads)
		{
			SetRetryIfFileIsLocked(yes);
			if (yes)
			{
				SetNumberOfRetryToOpenFile(numberOfThreads * 10);
				SetRetryTimeout(50);
			}
		}

		public static bool ThrowExceptionWhenInconsistencyFound()
		{
			return throwExceptionWhenInconsistencyFound;
		}

		public static void SetThrowExceptionWhenInconsistencyFound(bool throwExceptionWhenInconsistencyFound
			)
		{
			NeoDatis.Odb.OdbConfiguration.throwExceptionWhenInconsistencyFound = throwExceptionWhenInconsistencyFound;
		}

		public static bool AutomaticallyIncreaseCacheSize()
		{
			return automaticallyIncreaseCacheSize;
		}

		public static void SetAutomaticallyIncreaseCacheSize(bool automaticallyIncreaseCache
			)
		{
			automaticallyIncreaseCacheSize = automaticallyIncreaseCache;
		}

		public static int GetIdBlockSize()
		{
			return idBlockSize;
		}

		public static void SetIdBlockSize(int idBlockSize)
		{
			NeoDatis.Odb.OdbConfiguration.idBlockSize = idBlockSize;
		}

		public static int GetNB_IDS_PER_BLOCK()
		{
			return NbIdsPerBlock;
		}

		public static int GetID_BLOCK_REPETITION_SIZE()
		{
			return IdBlockRepetitionSize;
		}

		public static bool InPlaceUpdate()
		{
			return inPlaceUpdate;
		}

		/// <returns>Returns the stringSpaceReserveFactor.</returns>
		public static int GetStringSpaceReserveFactor()
		{
			return stringSpaceReserveFactor;
		}

		/// <param name="stringSpaceReserveFactor">The stringSpaceReserveFactor to set.</param>
		public static void SetStringSpaceReserveFactor(int stringSpaceReserveFactor)
		{
			NeoDatis.Odb.OdbConfiguration.stringSpaceReserveFactor = stringSpaceReserveFactor;
		}

		/// <returns>Returns the debugLevel.</returns>
		public static int GetDebugLevel()
		{
			return debugLevel;
		}

		/// <param name="debugLevel">The debugLevel to set.</param>
		public static void SetDebugLevel(int debugLevel)
		{
			NeoDatis.Odb.OdbConfiguration.debugLevel = debugLevel;
		}

		public static int GetDefaultBufferSizeForTransaction()
		{
			return defaultBufferSizeForTransaction;
		}

		public static void SetDefaultBufferSizeForTransaction(int defaultBufferSizeForTransaction
			)
		{
			NeoDatis.Odb.OdbConfiguration.defaultBufferSizeForTransaction = defaultBufferSizeForTransaction;
		}

		public static int GetNbBuffers()
		{
			return nbBuffers;
		}

		public static void SetNbBuffers(int nbBuffers)
		{
			NeoDatis.Odb.OdbConfiguration.nbBuffers = nbBuffers;
		}

		public static bool UseMultiBuffer()
		{
			return useMultiBuffer;
		}

		public static void SetUseMultiBuffer(bool useMultiBuffer)
		{
			NeoDatis.Odb.OdbConfiguration.useMultiBuffer = useMultiBuffer;
		}

		public static bool CheckModelCompatibility()
		{
			return checkModelCompatibility;
		}

		public static void SetCheckModelCompatibility(bool checkModelCompatibility)
		{
			NeoDatis.Odb.OdbConfiguration.checkModelCompatibility = checkModelCompatibility;
		}

		public static bool AutomaticCloseFileOnExit()
		{
			return automaticCloseFileOnExit;
		}

		public static void SetAutomaticCloseFileOnExit(bool automaticFileClose)
		{
			NeoDatis.Odb.OdbConfiguration.automaticCloseFileOnExit = automaticFileClose;
		}

		public static bool IsLogAll()
		{
			return logAll;
		}

		public static void SetLogAll(bool logAll)
		{
			NeoDatis.Odb.OdbConfiguration.logAll = logAll;
		}

		public static bool LogServerConnections()
		{
			return logServerConnections;
		}

		public static void SetLogServerConnections(bool logServerConnections)
		{
			NeoDatis.Odb.OdbConfiguration.logServerConnections = logServerConnections;
		}

		public static int GetDefaultIndexBTreeDegree()
		{
			return defaultIndexBTreeDegree;
		}

		public static void SetDefaultIndexBTreeDegree(int defaultIndexBTreeSize)
		{
			NeoDatis.Odb.OdbConfiguration.defaultIndexBTreeDegree = defaultIndexBTreeSize;
		}

		public static bool UseLazyCache()
		{
			return useLazyCache;
		}

		public static void SetUseLazyCache(bool useLazyCache)
		{
			NeoDatis.Odb.OdbConfiguration.useLazyCache = useLazyCache;
		}

		/// <returns>the queryExecutorCallback</returns>
		public static NeoDatis.Odb.Core.Query.Execution.IQueryExecutorCallback GetQueryExecutorCallback
			()
		{
			return queryExecutorCallback;
		}

		/// <param name="queryExecutorCallback">the queryExecutorCallback to set</param>
		public static void SetQueryExecutorCallback(NeoDatis.Odb.Core.Query.Execution.IQueryExecutorCallback
			 queryExecutorCallback)
		{
			NeoDatis.Odb.OdbConfiguration.queryExecutorCallback = queryExecutorCallback;
		}

		/// <returns>the useCache</returns>
		public static bool IsUseCache()
		{
			return useCache;
		}

		/// <param name="useCache">the useCache to set</param>
		public static void SetUseCache(bool useCache)
		{
			NeoDatis.Odb.OdbConfiguration.useCache = useCache;
		}

		public static bool IsMonitoringMemory()
		{
			return monitorMemory;
		}

		public static void MonitorMemory(bool yes)
		{
			monitorMemory = yes;
		}

		public static bool DisplayWarnings()
		{
			return displayWarnings;
		}

		public static void SetDisplayWarnings(bool yesOrNo)
		{
			displayWarnings = yesOrNo;
		}

		public static bool SaveHistory()
		{
			return saveHistory;
		}

		public static void SetSaveHistory(bool saveTheHistory)
		{
			saveHistory = saveTheHistory;
		}

		public static int GetScaleForAverageDivision()
		{
			return scaleForAverageDivision;
		}

		public static void SetScaleForAverageDivision(int scaleForAverageDivision)
		{
			NeoDatis.Odb.OdbConfiguration.scaleForAverageDivision = scaleForAverageDivision;
		}

		public static int GetRoundTypeForAverageDivision()
		{
			return roundTypeForAverageDivision;
		}

		public static void SetRoundTypeForAverageDivision(int roundTypeForAverageDivision
			)
		{
			NeoDatis.Odb.OdbConfiguration.roundTypeForAverageDivision = roundTypeForAverageDivision;
		}

		public static bool EnableEmptyConstructorCreation()
		{
			return enableEmptyConstructorCreation;
		}

		public static void SetEnableEmptyConstructorCreation(bool enableEmptyConstructorCreation
			)
		{
			NeoDatis.Odb.OdbConfiguration.enableEmptyConstructorCreation = enableEmptyConstructorCreation;
		}

		public static System.Type GetIOClass()
		{
			return ioClass;
		}

		public static void SetIOClass(System.Type IOClass, string password)
		{
			NeoDatis.Odb.OdbConfiguration.ioClass = IOClass;
			NeoDatis.Odb.OdbConfiguration.encryptionPassword = password;
		}

		public static string GetEncryptionPassword()
		{
			return encryptionPassword;
		}

		public static NeoDatis.Odb.Core.ICoreProvider GetCoreProvider()
		{
			if (!coreProviderInit)
			{
				coreProviderInit = true;
				try
				{
					coreProvider.Init2();
				}
				catch (System.Exception e)
				{
					throw new NeoDatis.Odb.ODBRuntimeException(NeoDatis.Odb.Core.NeoDatisError.ErrorInCoreProviderInitialization
						.AddParameter("Core Provider"), e);
				}
			}
			return coreProvider;
		}

		public static void SetCoreProvider(NeoDatis.Odb.Core.ICoreProvider coreProvider)
		{
			NeoDatis.Odb.OdbConfiguration.coreProvider = coreProvider;
		}

		public static string GetDatabaseCharacterEncoding()
		{
			return databaseCharacterEncoding;
		}

		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		public static void SetDatabaseCharacterEncoding(string dbCharacterEncoding)
		{
			if (dbCharacterEncoding != null)
			{
				// Checks if encoding is valid, using it in the String.getBytes
				// method
				new NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.DefaultByteArrayConverter().TestEncoding
					(dbCharacterEncoding);
				NeoDatis.Odb.OdbConfiguration.databaseCharacterEncoding = dbCharacterEncoding;
			}
			else
			{
				NeoDatis.Odb.OdbConfiguration.databaseCharacterEncoding = NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
					.NoEncoding;
			}
		}

		/// <exception cref="Java.IO.UnsupportedEncodingException"></exception>
		public static void SetLatinDatabaseCharacterEncoding()
		{
			NeoDatis.Odb.OdbConfiguration.databaseCharacterEncoding = defaultDatabaseCharacterEncoding;
		}

		public static bool HasEncoding()
		{
			return databaseCharacterEncoding != null && !databaseCharacterEncoding.Equals(NeoDatis.Odb.Impl.Core.Layers.Layer3.Engine.StorageEngineConstant
				.NoEncoding);
		}

		public static NeoDatis.Tool.Wrappers.ClassLoader GetClassLoader()
		{
			return classLoader;
		}

		public static void SetClassLoader(NeoDatis.Tool.Wrappers.ClassLoader classLoader)
		{
			NeoDatis.Odb.OdbConfiguration.classLoader = classLoader;
			NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassIntrospector().Reset();
			NeoDatis.Odb.OdbConfiguration.GetCoreProvider().GetClassPool().Reset();
		}

		public static bool CheckRuntimeVersion()
		{
			return checkRuntimeVersion;
		}

		public static void SetCheckRuntimeVersion(bool checkJavaRuntimeVersion)
		{
			NeoDatis.Odb.OdbConfiguration.checkRuntimeVersion = checkJavaRuntimeVersion;
		}

		/// <returns></returns>
		public static System.Type GetMessageStreamerClass()
		{
			return messageStreamerClass;
		}

		public static void SetMessageStreamerClass(System.Type messageStreamerClass)
		{
			NeoDatis.Odb.OdbConfiguration.messageStreamerClass = messageStreamerClass;
		}

		public static bool LogServerStartupAndShutdown()
		{
			return logServerStartupAndShutdown;
		}

		public static void SetLogServerStartupAndShutdown(bool logServerStartup)
		{
			NeoDatis.Odb.OdbConfiguration.logServerStartupAndShutdown = logServerStartup;
		}

		public static bool UseIndex()
		{
			return useIndex;
		}

		public static void SetUseIndex(bool useIndex)
		{
			NeoDatis.Odb.OdbConfiguration.useIndex = useIndex;
		}

		public static bool IsDebugEnabled()
		{
			return debugEnabled;
		}

		public static void SetDebugEnabled(bool debugEnabled)
		{
			NeoDatis.Odb.OdbConfiguration.debugEnabled = debugEnabled;
		}
	}
}
