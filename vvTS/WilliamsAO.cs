using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000074 RID: 116
	[HandlerCategory("vvWilliams"), HandlerName("Williams AO")]
	public class WilliamsAO : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x0600041D RID: 1053 RVA: 0x000162A8 File Offset: 0x000144A8
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("WilliamsAO", new string[]
			{
				this.PeriodFast.ToString(),
				this.PeriodSlow.ToString(),
				sec.get_CacheName()
			}, () => WilliamsAO.GenWilliamsAO(sec, this.Context, this.PeriodFast, this.PeriodSlow));
		}

		// Token: 0x0600041C RID: 1052 RVA: 0x00016128 File Offset: 0x00014328
		public static IList<double> GenWilliamsAO(ISecurity sec, IContext ctx, int _period1 = 5, int _period2 = 34)
		{
			int count = sec.get_Bars().Count;
			IList<double> arg_3D_0 = sec.get_ClosePrices();
			IList<double> medpr = ctx.GetData("MedianPrice", new string[]
			{
				sec.get_CacheName()
			}, () => Series.MedianPrice(sec.get_Bars()));
			double[] array = new double[count];
			IList<double> data = ctx.GetData("sma", new string[]
			{
				_period1.ToString(),
				medpr.GetHashCode().ToString()
			}, () => Series.SMA(medpr, _period1));
			IList<double> data2 = ctx.GetData("sma", new string[]
			{
				_period2.ToString(),
				medpr.GetHashCode().ToString()
			}, () => Series.SMA(medpr, _period2));
			for (int i = 0; i < count; i++)
			{
				array[i] = data[i] - data2[i];
			}
			return array;
		}

		// Token: 0x17000165 RID: 357
		public IContext Context
		{
			// Token: 0x0600041E RID: 1054 RVA: 0x0001631D File Offset: 0x0001451D
			get;
			// Token: 0x0600041F RID: 1055 RVA: 0x00016325 File Offset: 0x00014525
			set;
		}

		// Token: 0x17000163 RID: 355
		[HandlerParameter(true, "5", Min = "5", Max = "5", Step = "0")]
		public int PeriodFast
		{
			// Token: 0x06000418 RID: 1048 RVA: 0x000160C4 File Offset: 0x000142C4
			get;
			// Token: 0x06000419 RID: 1049 RVA: 0x000160CC File Offset: 0x000142CC
			set;
		}

		// Token: 0x17000164 RID: 356
		[HandlerParameter(true, "34", Min = "34", Max = "34", Step = "0")]
		public int PeriodSlow
		{
			// Token: 0x0600041A RID: 1050 RVA: 0x000160D5 File Offset: 0x000142D5
			get;
			// Token: 0x0600041B RID: 1051 RVA: 0x000160DD File Offset: 0x000142DD
			set;
		}
	}
}
