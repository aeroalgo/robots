using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000147 RID: 327
	[HandlerCategory("vvMACD")]
	public class MACDSig : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000A10 RID: 2576 RVA: 0x0002A299 File Offset: 0x00028499
		public IList<double> Execute(IList<double> src)
		{
			return EMA.GenEMA(src, this.Period);
		}

		// Token: 0x1700034B RID: 843
		[HandlerParameter(true, "9", Min = "3", Max = "20", Step = "1")]
		public int Period
		{
			// Token: 0x06000A0E RID: 2574 RVA: 0x0002A288 File Offset: 0x00028488
			get;
			// Token: 0x06000A0F RID: 2575 RVA: 0x0002A290 File Offset: 0x00028490
			set;
		}
	}
}
