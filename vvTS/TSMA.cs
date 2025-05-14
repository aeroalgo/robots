using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000198 RID: 408
	[HandlerCategory("vvAverages"), HandlerName("TSMA")]
	public class TSMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000CED RID: 3309 RVA: 0x00038EB0 File Offset: 0x000370B0
		public IList<double> Execute(IList<double> src)
		{
			return this.Context.GetData("tsma", new string[]
			{
				this.TsmaPeriod.ToString(),
				src.GetHashCode().ToString()
			}, () => TSMA.GenTSMA(src, this.Context, this.TsmaPeriod));
		}

		// Token: 0x06000CEC RID: 3308 RVA: 0x00038D1C File Offset: 0x00036F1C
		public static IList<double> GenTSMA(IList<double> src, IContext ctx, int period)
		{
			IList<double> sma1 = ctx.GetData("sma", new string[]
			{
				period.ToString(),
				src.GetHashCode().ToString()
			}, () => SMA.GenSMA(src, period));
			IList<double> sma2 = ctx.GetData("sma", new string[]
			{
				period.ToString(),
				sma1.GetHashCode().ToString()
			}, () => SMA.GenSMA(sma1, period));
			IList<double> data = ctx.GetData("sma", new string[]
			{
				period.ToString(),
				sma2.GetHashCode().ToString()
			}, () => SMA.GenSMA(sma2, period));
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = 3.0 * sma1[i] - 3.0 * sma2[i] + data[i];
			}
			return array;
		}

		// Token: 0x17000438 RID: 1080
		public IContext Context
		{
			// Token: 0x06000CEE RID: 3310 RVA: 0x00038F1C File Offset: 0x0003711C
			get;
			// Token: 0x06000CEF RID: 3311 RVA: 0x00038F24 File Offset: 0x00037124
			set;
		}

		// Token: 0x17000437 RID: 1079
		[HandlerParameter(true, "14", Min = "1", Max = "50", Step = "1")]
		public int TsmaPeriod
		{
			// Token: 0x06000CEA RID: 3306 RVA: 0x00038CC9 File Offset: 0x00036EC9
			get;
			// Token: 0x06000CEB RID: 3307 RVA: 0x00038CD1 File Offset: 0x00036ED1
			set;
		}
	}
}
