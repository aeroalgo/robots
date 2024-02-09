using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000060 RID: 96
	[HandlerCategory("vvIndicators"), HandlerName("Trix")]
	public class TRIX : BasePeriodIndicatorHandler, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000364 RID: 868 RVA: 0x000136A7 File Offset: 0x000118A7
		public IList<double> Execute(IList<double> src)
		{
			return TRIX.GenTRIX(src, base.get_Period(), this.Context);
		}

		// Token: 0x06000363 RID: 867 RVA: 0x00013530 File Offset: 0x00011730
		public static IList<double> GenTRIX(IList<double> src, int _period, IContext ctx)
		{
			IList<double> ema1 = ctx.GetData("ema", new string[]
			{
				_period.ToString(),
				src.GetHashCode().ToString()
			}, () => EMA.GenEMA(src, _period));
			IList<double> ema2 = ctx.GetData("ema", new string[]
			{
				_period.ToString(),
				ema1.GetHashCode().ToString()
			}, () => EMA.GenEMA(ema1, _period));
			IList<double> data = ctx.GetData("ema", new string[]
			{
				_period.ToString(),
				ema2.GetHashCode().ToString()
			}, () => EMA.GenEMA(ema2, _period));
			double[] array = new double[src.Count];
			for (int i = 1; i < src.Count; i++)
			{
				double num = data[i - 1];
				array[i] = ((num == 0.0) ? 0.0 : ((data[i] - num) / num * 100000.0));
			}
			return array;
		}

		// Token: 0x17000122 RID: 290
		public IContext Context
		{
			// Token: 0x06000365 RID: 869 RVA: 0x000136BB File Offset: 0x000118BB
			get;
			// Token: 0x06000366 RID: 870 RVA: 0x000136C3 File Offset: 0x000118C3
			set;
		}
	}
}
