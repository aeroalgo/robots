using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x020001A0 RID: 416
	[HandlerCategory("vvAverages"), HandlerName("TriMA")]
	public class TriMA : BasePeriodIndicatorHandler, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000D29 RID: 3369 RVA: 0x00039E88 File Offset: 0x00038088
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("trima", new string[]
			{
				base.get_Period().ToString(),
				src.GetHashCode().ToString()
			}, () => TriMA.GenTriMA(src, this.get_Period(), this.Context));
		}

		// Token: 0x06000D27 RID: 3367 RVA: 0x00039D58 File Offset: 0x00037F58
		public static IList<double> GenTriMA(IList<double> src, int period, IContext ctx)
		{
			IList<double> tri_ma = ctx.GetData("sma", new string[]
			{
				period.ToString(),
				src.GetHashCode().ToString()
			}, () => SMA.GenSMA(src, period));
			return ctx.GetData("sma", new string[]
			{
				period.ToString(),
				tri_ma.GetHashCode().ToString()
			}, () => SMA.GenSMA(tri_ma, period));
		}

		// Token: 0x06000D28 RID: 3368 RVA: 0x00039E0C File Offset: 0x0003800C
		public static double iTriMA(IList<double> price, int _period, int barNum)
		{
			int num = Convert.ToInt32(Math.Ceiling((double)(_period + 1) * 0.5));
			double num2 = 0.0;
			for (int i = 0; i < num; i++)
			{
				double num3 = SMA.iSMA(price, num, barNum - i);
				num2 += num3;
			}
			return num2 / (double)num;
		}

		// Token: 0x17000448 RID: 1096
		public IContext Context
		{
			// Token: 0x06000D2A RID: 3370 RVA: 0x00039EF4 File Offset: 0x000380F4
			get;
			// Token: 0x06000D2B RID: 3371 RVA: 0x00039EFC File Offset: 0x000380FC
			set;
		}
	}
}
