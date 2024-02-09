using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000075 RID: 117
	[HandlerCategory("vvWilliams"), HandlerName("Williams AC")]
	public class WilliamsAC : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x06000428 RID: 1064 RVA: 0x00016518 File Offset: 0x00014718
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("WilliamsAC", new string[]
			{
				this.PeriodFast.ToString(),
				this.PeriodSlow.ToString(),
				this.SmaPeriod.ToString(),
				sec.get_CacheName()
			}, () => WilliamsAC.GenWilliamsAC(sec, this.Context, this.PeriodFast, this.PeriodSlow, this.SmaPeriod));
		}

		// Token: 0x06000427 RID: 1063 RVA: 0x000163A4 File Offset: 0x000145A4
		public static IList<double> GenWilliamsAC(ISecurity sec, IContext ctx, int _Period1 = 5, int _Period2 = 34, int _SmaPeriod = 5)
		{
			IList<double> closePrices = sec.get_ClosePrices();
			double[] array = new double[closePrices.Count];
			IList<double> AwO = ctx.GetData("WilliamsAO", new string[]
			{
				_Period1.ToString(),
				_Period2.ToString(),
				sec.get_CacheName()
			}, () => WilliamsAO.GenWilliamsAO(sec, ctx, _Period1, _Period2));
			IList<double> data = ctx.GetData("sma", new string[]
			{
				_SmaPeriod.ToString(),
				AwO.GetHashCode().ToString()
			}, () => Series.SMA(AwO, _SmaPeriod));
			for (int i = 0; i < closePrices.Count; i++)
			{
				array[i] = AwO[i] - data[i];
			}
			return array;
		}

		// Token: 0x17000169 RID: 361
		public IContext Context
		{
			// Token: 0x06000429 RID: 1065 RVA: 0x0001659F File Offset: 0x0001479F
			get;
			// Token: 0x0600042A RID: 1066 RVA: 0x000165A7 File Offset: 0x000147A7
			set;
		}

		// Token: 0x17000166 RID: 358
		[HandlerParameter(true, "5", Min = "5", Max = "5", Step = "0")]
		public int PeriodFast
		{
			// Token: 0x06000421 RID: 1057 RVA: 0x00016336 File Offset: 0x00014536
			get;
			// Token: 0x06000422 RID: 1058 RVA: 0x0001633E File Offset: 0x0001453E
			set;
		}

		// Token: 0x17000167 RID: 359
		[HandlerParameter(true, "34", Min = "34", Max = "34", Step = "0")]
		public int PeriodSlow
		{
			// Token: 0x06000423 RID: 1059 RVA: 0x00016347 File Offset: 0x00014547
			get;
			// Token: 0x06000424 RID: 1060 RVA: 0x0001634F File Offset: 0x0001454F
			set;
		}

		// Token: 0x17000168 RID: 360
		[HandlerParameter(true, "5", Min = "5", Max = "5", Step = "0")]
		public int SmaPeriod
		{
			// Token: 0x06000425 RID: 1061 RVA: 0x00016358 File Offset: 0x00014558
			get;
			// Token: 0x06000426 RID: 1062 RVA: 0x00016360 File Offset: 0x00014560
			set;
		}
	}
}
