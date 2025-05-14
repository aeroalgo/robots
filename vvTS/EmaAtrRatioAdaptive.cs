using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000167 RID: 359
	[HandlerCategory("vvAverages"), HandlerName("Ema Atr Adaptive")]
	public class EmaAtrRatioAdaptive : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000B61 RID: 2913 RVA: 0x0002E8E8 File Offset: 0x0002CAE8
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("EmaATRadaptive", new string[]
			{
				this.AtrPeriod1.ToString(),
				this.AtrPeriod2.ToString(),
				sec.get_CacheName()
			}, () => EmaAtrRatioAdaptive.GenATRADAPTEMA(sec, this.AtrPeriod1, this.AtrPeriod2, this.Context));
		}

		// Token: 0x06000B60 RID: 2912 RVA: 0x0002E748 File Offset: 0x0002C948
		public static IList<double> GenATRADAPTEMA(ISecurity src, int period1, int period2, IContext ctx)
		{
			int count = src.get_Bars().Count;
			IList<double> closePrices = src.get_ClosePrices();
			double[] array = new double[count];
			IList<double> data = ctx.GetData("atr", new string[]
			{
				period1.ToString(),
				src.GetHashCode().ToString()
			}, () => ATR.ATR_TSLab(src.get_Bars(), period1));
			IList<double> data2 = ctx.GetData("atr", new string[]
			{
				period2.ToString(),
				src.GetHashCode().ToString()
			}, () => ATR.ATR_TSLab(src.get_Bars(), period2));
			for (int i = 1; i < count; i++)
			{
				if (i > period2)
				{
					double num = data[i] / data2[i];
					double num2 = 2.0 / (double)(period1 + 1);
					array[i] = num * num2 * closePrices[i] + (1.0 - num * num2) * array[i - 1];
				}
				else
				{
					array[i] = closePrices[i];
				}
			}
			return array;
		}

		// Token: 0x170003BE RID: 958
		[HandlerParameter(true, "50", Min = "1", Max = "60", Step = "1")]
		public int AtrPeriod1
		{
			// Token: 0x06000B5C RID: 2908 RVA: 0x0002E6EB File Offset: 0x0002C8EB
			get;
			// Token: 0x06000B5D RID: 2909 RVA: 0x0002E6F3 File Offset: 0x0002C8F3
			set;
		}

		// Token: 0x170003BF RID: 959
		[HandlerParameter(true, "150", Min = "1", Max = "200", Step = "1")]
		public int AtrPeriod2
		{
			// Token: 0x06000B5E RID: 2910 RVA: 0x0002E6FC File Offset: 0x0002C8FC
			get;
			// Token: 0x06000B5F RID: 2911 RVA: 0x0002E704 File Offset: 0x0002C904
			set;
		}

		// Token: 0x170003C0 RID: 960
		public IContext Context
		{
			// Token: 0x06000B62 RID: 2914 RVA: 0x0002E95D File Offset: 0x0002CB5D
			get;
			// Token: 0x06000B63 RID: 2915 RVA: 0x0002E965 File Offset: 0x0002CB65
			set;
		}
	}
}
