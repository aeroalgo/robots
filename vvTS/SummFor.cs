using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200010D RID: 269
	[HandlerCategory("vvTrade"), HandlerName("Сумма за")]
	public class SummFor : BasePeriodIndicatorHandler, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs
	{
		// Token: 0x06000790 RID: 1936 RVA: 0x0002133F File Offset: 0x0001F53F
		public IList<double> Execute(IList<double> src)
		{
			return this.SummaFor(src, base.get_Period());
		}

		// Token: 0x0600078F RID: 1935 RVA: 0x00021336 File Offset: 0x0001F536
		public IList<double> SummaFor(IList<double> src, int _period)
		{
			return Series.SummFor(src, _period);
		}
	}
}
