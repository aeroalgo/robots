using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000179 RID: 377
	[HandlerCategory("vvAverages"), HandlerName("HMAdn")]
	public class HullMAdn : BasePeriodIndicatorHandler, IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000BE9 RID: 3049 RVA: 0x000332A8 File Offset: 0x000314A8
		public IList<double> Execute(IList<double> src)
		{
			IList<double> list = this.GenHullMAdn(src, base.get_Period(), this.Context);
			IList<double> list2 = this.GenHullMAdn(src, base.get_Period() * 2, this.Context);
			double[] array = new double[src.Count];
			for (int i = 0; i < src.Count; i++)
			{
				array[i] = 2.0 * list[i] - list2[i];
			}
			return array;
		}

		// Token: 0x06000BE8 RID: 3048 RVA: 0x00033124 File Offset: 0x00031324
		public IList<double> GenHullMAdn(IList<double> src, int period, IContext context)
		{
			int per = period / 2;
			int period2 = Convert.ToInt32(Math.Sqrt((double)period));
			IList<double> list = new double[src.Count];
			IList<double> list2 = new double[src.Count];
			IList<double> data = context.GetData("lwma", new string[]
			{
				per.ToString(),
				src.GetHashCode().ToString()
			}, () => LWMA.GenWMA(src, per));
			IList<double> data2 = context.GetData("lwma", new string[]
			{
				period.ToString(),
				src.GetHashCode().ToString()
			}, () => LWMA.GenWMA(src, period));
			for (int i = 0; i < period; i++)
			{
				list2[i] = src[period];
			}
			for (int j = period; j < src.Count; j++)
			{
				list2[j] = 2.0 * data[j] - data2[j];
			}
			return LWMA.GenWMA(list2, period2);
		}

		// Token: 0x170003EA RID: 1002
		public IContext Context
		{
			// Token: 0x06000BEA RID: 3050 RVA: 0x00033319 File Offset: 0x00031519
			get;
			// Token: 0x06000BEB RID: 3051 RVA: 0x00033321 File Offset: 0x00031521
			set;
		}
	}
}
