using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000188 RID: 392
	[HandlerCategory("vvAverages"), HandlerName("MAMA")]
	public class MAMA : BaseAMA, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000C64 RID: 3172 RVA: 0x00035D89 File Offset: 0x00033F89
		public IList<double> Execute(IList<double> src)
		{
			return base.Execute(src, true);
		}
	}
}
