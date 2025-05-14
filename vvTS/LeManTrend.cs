using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000033 RID: 51
	[HandlerCategory("vvIndicators"), HandlerName("LeMan Trend")]
	public class LeManTrend : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060001D3 RID: 467 RVA: 0x00008E08 File Offset: 0x00007008
		public IList<double> Execute(ISecurity sec)
		{
			return this.Context.GetData("lemantrend", new string[]
			{
				this.MinPeriod.ToString(),
				this.MiddlePeriod.ToString(),
				this.MaxPeriod.ToString(),
				this.smooth.ToString(),
				this.Down.ToString(),
				sec.get_CacheName()
			}, () => LeManTrend.GenLeManTrend(sec, this.Context, this.MinPeriod, this.MiddlePeriod, this.MaxPeriod, this.smooth, this.Down));
		}

		// Token: 0x060001D2 RID: 466 RVA: 0x00008A5C File Offset: 0x00006C5C
		public static IList<double> GenLeManTrend(ISecurity sec, IContext ctx, int _MinPeriod, int _MiddlePeriod, int _MaxPeriod, int _smooth, bool _Down)
		{
			int count = sec.get_Bars().Count;
			IList<double> High = sec.get_HighPrices();
			IList<double> Low = sec.get_LowPrices();
			double[] TempBuffer1 = new double[count];
			double[] TempBuffer2 = new double[count];
			IList<double> data = ctx.GetData("hhv", new string[]
			{
				_MinPeriod.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(High, _MinPeriod));
			IList<double> data2 = ctx.GetData("hhv", new string[]
			{
				_MiddlePeriod.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(High, _MiddlePeriod));
			IList<double> data3 = ctx.GetData("hhv", new string[]
			{
				_MaxPeriod.ToString(),
				sec.get_CacheName()
			}, () => Series.Highest(High, _MaxPeriod));
			IList<double> data4 = ctx.GetData("llv", new string[]
			{
				_MinPeriod.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(Low, _MinPeriod));
			IList<double> data5 = ctx.GetData("llv", new string[]
			{
				_MiddlePeriod.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(Low, _MiddlePeriod));
			IList<double> data6 = ctx.GetData("llv", new string[]
			{
				_MaxPeriod.ToString(),
				sec.get_CacheName()
			}, () => Series.Lowest(Low, _MaxPeriod));
			for (int i = 0; i < count; i++)
			{
				TempBuffer1[i] = High[i] - data[i] + (High[i] - data2[i]) + (High[i] - data3[i]);
				TempBuffer2[i] = data4[i] - Low[i] + (data5[i] - Low[i]) + (data6[i] - Low[i]);
			}
			IList<double> data7 = ctx.GetData("JsMA", new string[]
			{
				_smooth.ToString(),
				TempBuffer1.GetHashCode().ToString()
			}, () => JsMA.GenJsMA(TempBuffer1, _smooth));
			IList<double> data8 = ctx.GetData("JsMA", new string[]
			{
				_smooth.ToString(),
				TempBuffer2.GetHashCode().ToString()
			}, () => JsMA.GenJsMA(TempBuffer2, _smooth));
			if (!_Down)
			{
				return data7;
			}
			return data8;
		}

		// Token: 0x1700009D RID: 157
		public IContext Context
		{
			// Token: 0x060001D4 RID: 468 RVA: 0x00008EB3 File Offset: 0x000070B3
			get;
			// Token: 0x060001D5 RID: 469 RVA: 0x00008EBB File Offset: 0x000070BB
			set;
		}

		// Token: 0x1700009C RID: 156
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Down
		{
			// Token: 0x060001D0 RID: 464 RVA: 0x000089A9 File Offset: 0x00006BA9
			get;
			// Token: 0x060001D1 RID: 465 RVA: 0x000089B1 File Offset: 0x00006BB1
			set;
		}

		// Token: 0x1700009A RID: 154
		[HandlerParameter(true, "34", Min = "5", Max = "50", Step = "1")]
		public int MaxPeriod
		{
			// Token: 0x060001CC RID: 460 RVA: 0x00008987 File Offset: 0x00006B87
			get;
			// Token: 0x060001CD RID: 461 RVA: 0x0000898F File Offset: 0x00006B8F
			set;
		}

		// Token: 0x17000099 RID: 153
		[HandlerParameter(true, "21", Min = "5", Max = "50", Step = "1")]
		public int MiddlePeriod
		{
			// Token: 0x060001CA RID: 458 RVA: 0x00008976 File Offset: 0x00006B76
			get;
			// Token: 0x060001CB RID: 459 RVA: 0x0000897E File Offset: 0x00006B7E
			set;
		}

		// Token: 0x17000098 RID: 152
		[HandlerParameter(true, "13", Min = "5", Max = "50", Step = "1")]
		public int MinPeriod
		{
			// Token: 0x060001C8 RID: 456 RVA: 0x00008965 File Offset: 0x00006B65
			get;
			// Token: 0x060001C9 RID: 457 RVA: 0x0000896D File Offset: 0x00006B6D
			set;
		}

		// Token: 0x1700009B RID: 155
		[HandlerParameter(true, "3", Min = "3", Max = "10", Step = "1")]
		public int smooth
		{
			// Token: 0x060001CE RID: 462 RVA: 0x00008998 File Offset: 0x00006B98
			get;
			// Token: 0x060001CF RID: 463 RVA: 0x000089A0 File Offset: 0x00006BA0
			set;
		}
	}
}
