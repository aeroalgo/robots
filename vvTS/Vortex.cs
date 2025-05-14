using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x0200006B RID: 107
	[HandlerCategory("vvIndicators"), HandlerName("Vortex")]
	public class Vortex : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs, IContextUses
	{
		// Token: 0x060003C0 RID: 960 RVA: 0x00014C95 File Offset: 0x00012E95
		public IList<double> Execute(ISecurity src)
		{
			return Vortex.GenVortex(src, this.Context, this.Period, this.preSmooth, this.postSmooth, this.Chart, this.Output);
		}

		// Token: 0x060003BF RID: 959 RVA: 0x000149BC File Offset: 0x00012BBC
		public static IList<double> GenVortex(ISecurity src, IContext ctx, int period, int presmooth, int postsmooth, int _Chart, int _Output)
		{
			int count = src.get_Bars().Count;
			if (count < period)
			{
				return null;
			}
			IList<double> list = src.get_ClosePrices();
			if (presmooth > 0)
			{
				list = JMA.GenJMA(list, presmooth, 100);
			}
			IList<double> arg_2D_0 = src.get_OpenPrices();
			IList<double> highPrices = src.get_HighPrices();
			IList<double> lowPrices = src.get_LowPrices();
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			double[] array5 = new double[count];
			double[] array6 = new double[count];
			for (int i = period; i < count; i++)
			{
				array5[i] = Math.Max(highPrices[i], list[i - 1]) - Math.Min(lowPrices[i], list[i - 1]);
				array3[i] = Math.Abs(highPrices[i] - lowPrices[i - 1]);
				array4[i] = Math.Abs(lowPrices[i] - highPrices[i - 1]);
				array[i] = 0.0;
				array2[i] = 0.0;
				double num = 0.0;
				double num2 = 0.0;
				double num3 = 0.0;
				for (int j = 0; j < period; j++)
				{
					num += array3[i - j];
					num2 += array4[i - j];
					num3 += array5[i - j];
				}
				if (num3 != 0.0)
				{
					array[i] = num / num3;
					array2[i] = num2 / num3;
				}
			}
			IList<double> list2 = array2;
			IList<double> list3 = array;
			if (postsmooth > 0)
			{
				list2 = JMA.GenJMA(array2, postsmooth, 100);
				list3 = JMA.GenJMA(array, postsmooth, 100);
			}
			if (_Output == 2)
			{
				for (int k = 1; k < count; k++)
				{
					array6[k] = 0.0;
					if (list3[k] > list2[k])
					{
						array6[k] = 1.0;
					}
					if (list3[k] < list2[k])
					{
						array6[k] = -1.0;
					}
				}
			}
			if (_Chart > 0)
			{
				IPane pane = ctx.CreatePane("", 30.0, false, false);
				pane.AddList(string.Concat(new string[]
				{
					"Vortex(",
					period.ToString(),
					",",
					presmooth.ToString(),
					",",
					postsmooth.ToString(),
					")"
				}), list3, 0, 294916, 0, 0);
				pane.AddList("", list2, 0, 13702664, 0, 0);
			}
			if (_Output == 2)
			{
				return array6;
			}
			if (_Output == 1)
			{
				return list2;
			}
			return list3;
		}

		// Token: 0x17000141 RID: 321
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1")]
		public int Chart
		{
			// Token: 0x060003BB RID: 955 RVA: 0x00014998 File Offset: 0x00012B98
			get;
			// Token: 0x060003BC RID: 956 RVA: 0x000149A0 File Offset: 0x00012BA0
			set;
		}

		// Token: 0x17000143 RID: 323
		public IContext Context
		{
			// Token: 0x060003C1 RID: 961 RVA: 0x00014CC1 File Offset: 0x00012EC1
			get;
			// Token: 0x060003C2 RID: 962 RVA: 0x00014CC9 File Offset: 0x00012EC9
			set;
		}

		// Token: 0x17000142 RID: 322
		[HandlerParameter(true, "0", Min = "0", Max = "2", Step = "1", Name = "0-plusVI\n1-minusVI\n2-signals")]
		public int Output
		{
			// Token: 0x060003BD RID: 957 RVA: 0x000149A9 File Offset: 0x00012BA9
			get;
			// Token: 0x060003BE RID: 958 RVA: 0x000149B1 File Offset: 0x00012BB1
			set;
		}

		// Token: 0x1700013E RID: 318
		[HandlerParameter(true, "14", Min = "3", Max = "60", Step = "1")]
		public int Period
		{
			// Token: 0x060003B5 RID: 949 RVA: 0x00014965 File Offset: 0x00012B65
			get;
			// Token: 0x060003B6 RID: 950 RVA: 0x0001496D File Offset: 0x00012B6D
			set;
		}

		// Token: 0x17000140 RID: 320
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int postSmooth
		{
			// Token: 0x060003B9 RID: 953 RVA: 0x00014987 File Offset: 0x00012B87
			get;
			// Token: 0x060003BA RID: 954 RVA: 0x0001498F File Offset: 0x00012B8F
			set;
		}

		// Token: 0x1700013F RID: 319
		[HandlerParameter(true, "0", Min = "0", Max = "20", Step = "1")]
		public int preSmooth
		{
			// Token: 0x060003B7 RID: 951 RVA: 0x00014976 File Offset: 0x00012B76
			get;
			// Token: 0x060003B8 RID: 952 RVA: 0x0001497E File Offset: 0x00012B7E
			set;
		}
	}
}
