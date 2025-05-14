using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x0200002F RID: 47
	[HandlerCategory("vvIndicators"), HandlerDecimals(2), HandlerName("H-L Volatility")]
	public class HLVolty : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001AD RID: 429 RVA: 0x000082D4 File Offset: 0x000064D4
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("HLvolatility", new string[]
			{
				this.Period.ToString(),
				this.Smooth.ToString(),
				sec.get_CacheName()
			}, () => HLVolty.GenHLVolty(sec, this.Context, this.Period, this.Smooth));
		}

		// Token: 0x060001AC RID: 428 RVA: 0x00008188 File Offset: 0x00006388
		public static IList<double> GenHLVolty(ISecurity sec, IContext ctx, int period, int smooth)
		{
			int count = sec.get_Bars().Count;
			IList<double> arg_2D_0 = sec.get_ClosePrices();
			IList<double> highPrices = sec.get_HighPrices();
			IList<double> lowPrices = sec.get_LowPrices();
			IList<double> arg_54_0 = sec.get_OpenPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			IList<double> data = ctx.GetData("MedianPrice", new string[]
			{
				sec.get_CacheName()
			}, () => Series.MedianPrice(sec.get_Bars()));
			for (int i = 1; i < count; i++)
			{
				array2[i] = highPrices[i] - lowPrices[i];
				array3[i] = EMA.iEMA(array2, array3, period, i);
				array[i] = 100.0 * (array3[i] / data[i]);
			}
			IList<double> result = array;
			if (smooth > 0)
			{
				result = JMA.GenJMA(array, smooth, 100);
			}
			return result;
		}

		// Token: 0x17000090 RID: 144
		public IContext Context
		{
			// Token: 0x060001AE RID: 430 RVA: 0x00008349 File Offset: 0x00006549
			get;
			// Token: 0x060001AF RID: 431 RVA: 0x00008351 File Offset: 0x00006551
			set;
		}

		// Token: 0x1700008E RID: 142
		[HandlerParameter(true, "14", Min = "9", Max = "30", Step = "1")]
		public int Period
		{
			// Token: 0x060001A8 RID: 424 RVA: 0x0000814C File Offset: 0x0000634C
			get;
			// Token: 0x060001A9 RID: 425 RVA: 0x00008154 File Offset: 0x00006354
			set;
		}

		// Token: 0x1700008F RID: 143
		[HandlerParameter(true, "0", Min = "0", Max = "15", Step = "1")]
		public int Smooth
		{
			// Token: 0x060001AA RID: 426 RVA: 0x0000815D File Offset: 0x0000635D
			get;
			// Token: 0x060001AB RID: 427 RVA: 0x00008165 File Offset: 0x00006365
			set;
		}
	}
}
