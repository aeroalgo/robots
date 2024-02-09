using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000139 RID: 313
	[HandlerCategory("vvRSI"), HandlerDecimals(2), HandlerName("LaguerreRSI&Filter2")]
	public class LaguerreRSInFilter2 : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000974 RID: 2420 RVA: 0x00027AD9 File Offset: 0x00025CD9
		public IList<double> Execute(IList<double> src)
		{
			return this.GenLaguerreRSInFilter(src, this.RsiGamma, this.RsiSmooth);
		}

		// Token: 0x06000973 RID: 2419 RVA: 0x00027918 File Offset: 0x00025B18
		public IList<double> GenLaguerreFilter(IList<double> _source, double _gamma)
		{
			double[] array = new double[_source.Count];
			double[] array2 = new double[_source.Count];
			double[] array3 = new double[_source.Count];
			double[] array4 = new double[_source.Count];
			double[] array5 = new double[_source.Count];
			for (int i = 1; i < _source.Count; i++)
			{
				array2[i] = (1.0 - _gamma) * _source[i] + _gamma * array2[i - 1];
				array3[i] = -_gamma * array2[i] + array2[i - 1] + _gamma * array3[i - 1];
				array4[i] = -_gamma * array3[i] + array3[i - 1] + _gamma * array4[i - 1];
				array5[i] = -_gamma * array4[i] + array4[i - 1] + _gamma * array5[i - 1];
				double num = 0.0;
				double num2 = 0.0;
				if (array2[i] >= array3[i])
				{
					num = array2[i] - array3[i];
				}
				else
				{
					num2 = array3[i] - array2[i];
				}
				if (array3[i] >= array4[i])
				{
					num = num + array3[i] - array4[i];
				}
				else
				{
					num2 = num2 + array4[i] - array3[i];
				}
				if (array4[i] >= array5[i])
				{
					num = num + array4[i] - array5[i];
				}
				else
				{
					num2 = num2 + array5[i] - array4[i];
				}
				if (num + num2 != 0.0)
				{
					array[i] = (array2[i] + 2.0 * array3[i] + 2.0 * array4[i] + array5[i]) / 6.0;
				}
			}
			return array;
		}

		// Token: 0x06000972 RID: 2418 RVA: 0x000274D0 File Offset: 0x000256D0
		public IList<double> GenLaguerreRSInFilter(IList<double> src, double _gamma, int _smooth)
		{
			double[] array = new double[src.Count];
			double[] array2 = new double[src.Count];
			double[] array3 = new double[src.Count];
			double[] array4 = new double[src.Count];
			double[] array5 = new double[src.Count];
			double[] array6 = new double[src.Count];
			IList<double> list = Series.SMA(src, _smooth);
			for (int i = 1; i < src.Count; i++)
			{
				array3[i] = (1.0 - _gamma) * list[i] + _gamma * array3[i - 1];
				array4[i] = -_gamma * array3[i] + array3[i - 1] + _gamma * array4[i - 1];
				array5[i] = -_gamma * array4[i] + array4[i - 1] + _gamma * array5[i - 1];
				array6[i] = -_gamma * array5[i] + array5[i - 1] + _gamma * array6[i - 1];
				double num = 0.0;
				double num2 = 0.0;
				if (array3[i] >= array4[i])
				{
					num = array3[i] - array4[i];
				}
				else
				{
					num2 = array4[i] - array3[i];
				}
				if (array4[i] >= array5[i])
				{
					num = num + array4[i] - array5[i];
				}
				else
				{
					num2 = num2 + array5[i] - array4[i];
				}
				if (array5[i] >= array6[i])
				{
					num = num + array5[i] - array6[i];
				}
				else
				{
					num2 = num2 + array6[i] - array5[i];
				}
				if (num + num2 != 0.0)
				{
					array[i] = num / (num + num2);
				}
			}
			IList<double> list2 = this.GenLaguerreFilter(array, this.FilterGamma);
			if (this.TradeSignals)
			{
				for (int j = 1; j < src.Count; j++)
				{
					array2[j] = 0.0;
					double num3 = array[j];
					double num4 = array[j - 1];
					double num5 = list2[j];
					double num6 = list2[j - 1];
					if (num3 > num5 && num4 < num6 && num3 < this.NoTradeLvlDown)
					{
						array2[j] = 1.0;
					}
					if (num3 < num5 && num4 > num6 && num3 > this.NoTradeLvlUp)
					{
						array2[j] = -1.0;
					}
				}
			}
			if (this.Chart)
			{
				IPane pane = this.Context.CreatePane("LaguerreRSI&Filter", 40.0, false, false);
				pane.AddList(string.Concat(new string[]
				{
					"LagRSI(",
					this.RsiGamma.ToString(),
					",",
					this.RsiSmooth.ToString(),
					")"
				}), array, 0, 329128, 0, 0);
				pane.AddList("Filter(" + this.FilterGamma.ToString() + ")", list2, 0, 13043219, 0, 0);
				IList<double> list3 = Line.GenLine(src, this.LevelUp);
				IList<double> list4 = Line.GenLine(src, this.LevelDown);
				IList<double> list5 = Line.GenLine(src, this.NoTradeLvlUp);
				IList<double> list6 = Line.GenLine(src, this.NoTradeLvlDown);
				pane.AddList("LevelUp(" + this.LevelUp.ToString() + ")", list3, 0, 237840, 2, 0);
				pane.AddList("LevelDown(" + this.LevelDown.ToString() + ")", list4, 0, 237840, 2, 0);
				pane.AddList("NoTradeUp(" + this.NoTradeLvlUp.ToString() + ")", list5, 0, 186837, 1, 0);
				pane.AddList("LevelDown(" + this.NoTradeLvlDown.ToString() + ")", list6, 0, 186837, 1, 0);
			}
			if (this.TradeSignals)
			{
				return array2;
			}
			if (!this.DrawFilter)
			{
				return array;
			}
			return list2;
		}

		// Token: 0x1700030C RID: 780
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool Chart
		{
			// Token: 0x06000966 RID: 2406 RVA: 0x00027469 File Offset: 0x00025669
			get;
			// Token: 0x06000967 RID: 2407 RVA: 0x00027471 File Offset: 0x00025671
			set;
		}

		// Token: 0x17000312 RID: 786
		public IContext Context
		{
			// Token: 0x06000975 RID: 2421 RVA: 0x00027AEE File Offset: 0x00025CEE
			get;
			// Token: 0x06000976 RID: 2422 RVA: 0x00027AF6 File Offset: 0x00025CF6
			set;
		}

		// Token: 0x1700030B RID: 779
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool DrawFilter
		{
			// Token: 0x06000964 RID: 2404 RVA: 0x00027458 File Offset: 0x00025658
			get;
			// Token: 0x06000965 RID: 2405 RVA: 0x00027460 File Offset: 0x00025660
			set;
		}

		// Token: 0x1700030A RID: 778
		[HandlerParameter(true, "0.60", Min = "0.05", Max = "1", Step = "0.05")]
		public double FilterGamma
		{
			// Token: 0x06000962 RID: 2402 RVA: 0x00027447 File Offset: 0x00025647
			get;
			// Token: 0x06000963 RID: 2403 RVA: 0x0002744F File Offset: 0x0002564F
			set;
		}

		// Token: 0x1700030F RID: 783
		[HandlerParameter(true, "0.15", Min = "0.05", Max = "1", Step = "0.05")]
		public double LevelDown
		{
			// Token: 0x0600096C RID: 2412 RVA: 0x0002749C File Offset: 0x0002569C
			get;
			// Token: 0x0600096D RID: 2413 RVA: 0x000274A4 File Offset: 0x000256A4
			set;
		}

		// Token: 0x1700030E RID: 782
		[HandlerParameter(true, "0.85", Min = "0.05", Max = "1", Step = "0.05")]
		public double LevelUp
		{
			// Token: 0x0600096A RID: 2410 RVA: 0x0002748B File Offset: 0x0002568B
			get;
			// Token: 0x0600096B RID: 2411 RVA: 0x00027493 File Offset: 0x00025693
			set;
		}

		// Token: 0x17000311 RID: 785
		[HandlerParameter(true, "0.35", Min = "0.05", Max = "1", Step = "0.05")]
		public double NoTradeLvlDown
		{
			// Token: 0x06000970 RID: 2416 RVA: 0x000274BE File Offset: 0x000256BE
			get;
			// Token: 0x06000971 RID: 2417 RVA: 0x000274C6 File Offset: 0x000256C6
			set;
		}

		// Token: 0x17000310 RID: 784
		[HandlerParameter(true, "0.65", Min = "0.05", Max = "1", Step = "0.05")]
		public double NoTradeLvlUp
		{
			// Token: 0x0600096E RID: 2414 RVA: 0x000274AD File Offset: 0x000256AD
			get;
			// Token: 0x0600096F RID: 2415 RVA: 0x000274B5 File Offset: 0x000256B5
			set;
		}

		// Token: 0x17000308 RID: 776
		[HandlerParameter(true, "0.60", Min = "0.05", Max = "1", Step = "0.1")]
		public double RsiGamma
		{
			// Token: 0x0600095E RID: 2398 RVA: 0x00027425 File Offset: 0x00025625
			get;
			// Token: 0x0600095F RID: 2399 RVA: 0x0002742D File Offset: 0x0002562D
			set;
		}

		// Token: 0x17000309 RID: 777
		[HandlerParameter(true, "1", Min = "1", Max = "5", Step = "1")]
		public int RsiSmooth
		{
			// Token: 0x06000960 RID: 2400 RVA: 0x00027436 File Offset: 0x00025636
			get;
			// Token: 0x06000961 RID: 2401 RVA: 0x0002743E File Offset: 0x0002563E
			set;
		}

		// Token: 0x1700030D RID: 781
		[HandlerParameter(false, "false", NotOptimized = true)]
		public bool TradeSignals
		{
			// Token: 0x06000968 RID: 2408 RVA: 0x0002747A File Offset: 0x0002567A
			get;
			// Token: 0x06000969 RID: 2409 RVA: 0x00027482 File Offset: 0x00025682
			set;
		}
	}
}
