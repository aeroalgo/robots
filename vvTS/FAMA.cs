using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000172 RID: 370
	[HandlerCategory("vvAverages"), HandlerName("FAMA")]
	public class FAMA : BaseAMA, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000BAA RID: 2986 RVA: 0x000321D8 File Offset: 0x000303D8
		public IList<double> Execute(IList<double> src)
		{
			return base.Execute(src, false);
		}
	}
}
