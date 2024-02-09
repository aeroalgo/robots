using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000144 RID: 324
	[HandlerCategory("vvMACD"), HandlerName("LeaderMACD")]
	public class LeaderMACD : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000A02 RID: 2562 RVA: 0x00029E24 File Offset: 0x00028024
		public IList<double> Execute(IList<double> src)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			double[] array7 = new double[count];
			for (int i = 1; i < count; i++)
			{
				array3[i] = EMA.iEMA(src, array3, this.FastEMA, i);
				array4[i] = EMA.iEMA(src, array4, this.FastEMA, i);
				array5[i] = src[i] - array3[i];
				array6[i] = src[i] - array4[i];
			}
			IList<double> list = EMA.GenEMA(array5, this.FastEMA);
			IList<double> list2 = EMA.GenEMA(array6, this.SlowEMA);
			for (int j = 0; j < count; j++)
			{
				array[j] = array3[j] + list[j] - (array4[j] + list2[j]);
			}
			IList<double> list3 = EMA.GenEMA(array, this.SignalEMA);
			for (int k = 1; k < count; k++)
			{
				array2[k] = array[k] - list3[k];
				array7[k] = 0.0;
				if (array[k] > array2[k] && array[k - 1] < array2[k - 1])
				{
					array7[k] = 1.0;
				}
				if (array[k] < array2[k] && array[k - 1] > array2[k - 1])
				{
					array7[k] = -1.0;
				}
			}
			if (this.Chart > 0)
			{
				IPane pane = this.Context.CreatePane("LeaderMACD", 40.0, false, false);
				pane.AddList(string.Concat(new string[]
				{
					"LM(",
					this.FastEMA.ToString(),
					",",
					this.SlowEMA.ToString(),
					",",
					this.SignalEMA.ToString(),
					")"
				}), array, 0, 329118, 0, 0);
				pane.AddList("signal(" + this.SignalEMA.ToString() + ")", array2, 0, 13369892, 0, 0);
			}
			if (this.Output == 1)
			{
				return array2;
			}
			if (this.Output == 2)
			{
				return array7;
			}
			return array;
		}

		// Token: 0x17000348 RID: 840
		[HandlerParameter(true, "0", Min = "0", Max = "3", Step = "1")]
		public int Chart
		{
			// Token: 0x06000A00 RID: 2560 RVA: 0x00029E12 File Offset: 0x00028012
			get;
			// Token: 0x06000A01 RID: 2561 RVA: 0x00029E1A File Offset: 0x0002801A
			set;
		}

		// Token: 0x17000349 RID: 841
		public IContext Context
		{
			// Token: 0x06000A03 RID: 2563 RVA: 0x0002A0A2 File Offset: 0x000282A2
			get;
			// Token: 0x06000A04 RID: 2564 RVA: 0x0002A0AA File Offset: 0x000282AA
			set;
		}

		// Token: 0x17000344 RID: 836
		[HandlerParameter(true, "12", Min = "3", Max = "20", Step = "1")]
		public int FastEMA
		{
			// Token: 0x060009F8 RID: 2552 RVA: 0x00029DCE File Offset: 0x00027FCE
			get;
			// Token: 0x060009F9 RID: 2553 RVA: 0x00029DD6 File Offset: 0x00027FD6
			set;
		}

		// Token: 0x17000347 RID: 839
		[HandlerParameter(true, "0", Min = "0", Max = "3", Step = "1")]
		public int Output
		{
			// Token: 0x060009FE RID: 2558 RVA: 0x00029E01 File Offset: 0x00028001
			get;
			// Token: 0x060009FF RID: 2559 RVA: 0x00029E09 File Offset: 0x00028009
			set;
		}

		// Token: 0x17000346 RID: 838
		[HandlerParameter(true, "9", Min = "3", Max = "20", Step = "1")]
		public int SignalEMA
		{
			// Token: 0x060009FC RID: 2556 RVA: 0x00029DF0 File Offset: 0x00027FF0
			get;
			// Token: 0x060009FD RID: 2557 RVA: 0x00029DF8 File Offset: 0x00027FF8
			set;
		}

		// Token: 0x17000345 RID: 837
		[HandlerParameter(true, "26", Min = "3", Max = "20", Step = "1")]
		public int SlowEMA
		{
			// Token: 0x060009FA RID: 2554 RVA: 0x00029DDF File Offset: 0x00027FDF
			get;
			// Token: 0x060009FB RID: 2555 RVA: 0x00029DE7 File Offset: 0x00027FE7
			set;
		}
	}
}
