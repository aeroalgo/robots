using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000173 RID: 371
	[HandlerCategory("vvAverages"), HandlerName("FRAMA")]
	public class FRAMA : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000BAF RID: 2991 RVA: 0x00032518 File Offset: 0x00030718
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("FRAMA", new string[]
			{
				this.Period.ToString(),
				sec.GetHashCode().ToString()
			}, () => this.GenFRAMA(sec, this.Period, this.Context));
		}

		// Token: 0x06000BAE RID: 2990 RVA: 0x00032264 File Offset: 0x00030464
		public IList<double> GenFRAMA(ISecurity sec, int period, IContext context)
		{
			IList<double> data = context.GetData("hhv", new string[]
			{
				period.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), period));
			IList<double> data2 = context.GetData("llv", new string[]
			{
				period.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), period));
			int periodx2 = period * 2;
			IList<double> data3 = context.GetData("hhv", new string[]
			{
				periodx2.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(sec.get_HighPrices(), periodx2));
			IList<double> data4 = context.GetData("llv", new string[]
			{
				periodx2.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(sec.get_LowPrices(), periodx2));
			double[] array = new double[sec.get_Bars().Count];
			for (int i = 2 * period; i < sec.get_Bars().Count; i++)
			{
				double num = data[i];
				double num2 = data2[i];
				double num3 = data[i - period];
				double num4 = data2[i - period];
				double num5 = data3[i];
				double num6 = data4[i];
				double num7 = (num - num2) / (double)period;
				double num8 = (num3 - num4) / (double)period;
				double d = (num5 - num6) / (2.0 * (double)period);
				double num9 = (Math.Log(num7 + num8) - Math.Log(d)) / Math.Log(2.0);
				double num10 = Math.Exp(-4.6 * (num9 - 1.0));
				array[i] = num10 * sec.get_ClosePrices()[i] + (1.0 - num10) * array[i - 1];
			}
			return array;
		}

		// Token: 0x170003D8 RID: 984
		public IContext Context
		{
			// Token: 0x06000BB0 RID: 2992 RVA: 0x00032584 File Offset: 0x00030784
			get;
			// Token: 0x06000BB1 RID: 2993 RVA: 0x0003258C File Offset: 0x0003078C
			set;
		}

		// Token: 0x170003D7 RID: 983
		[HandlerParameter(true, "10", Min = "1", Max = "50", Step = "1")]
		public int Period
		{
			// Token: 0x06000BAC RID: 2988 RVA: 0x000321EA File Offset: 0x000303EA
			get;
			// Token: 0x06000BAD RID: 2989 RVA: 0x000321F2 File Offset: 0x000303F2
			set;
		}
	}
}
