using System;
using System.Collections.Generic;
using TSLab.Script;
using TSLab.Script.Handlers;

namespace vvTSLtools
{
	// Token: 0x02000011 RID: 17
	[HandlerCategory("vvIndicators"), HandlerName("AroonOsc")]
	public class AroonOsc : IBar2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, ISecurityInputs
	{
		// Token: 0x06000091 RID: 145 RVA: 0x00003ACC File Offset: 0x00001CCC
		public IList<double> Execute(ISecurity src)
		{
			return AroonOsc.GenAroonOsc(src, this.AroonPeriod, this.Filter, this.Output);
		}

		// Token: 0x06000090 RID: 144 RVA: 0x000039FC File Offset: 0x00001BFC
		public static IList<double> GenAroonOsc(ISecurity src, int _AroonPeriod, int _Filter, int _Output)
		{
			int count = src.get_Bars().Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			for (int i = 1; i < count; i++)
			{
				double num = 100.0 * ((double)_AroonPeriod - vvSeries.HighestBarNum(src.get_HighPrices(), i, _AroonPeriod)) / (double)_AroonPeriod;
				double num2 = 100.0 * ((double)_AroonPeriod - vvSeries.LowestBarNum(src.get_LowPrices(), i, _AroonPeriod)) / (double)_AroonPeriod;
				array[i] = num - num2;
				array2[i] = array2[i - 1];
				if (array[i] > (double)_Filter)
				{
					array2[i] = 1.0;
				}
				if (array[i] < (double)(-(double)_Filter))
				{
					array2[i] = -1.0;
				}
				if (array[i] < (double)_Filter && array[i] > (double)(-(double)_Filter))
				{
					array2[i] = 0.0;
				}
			}
			if (_Output == 1)
			{
				return array2;
			}
			return array;
		}

		// Token: 0x1700002C RID: 44
		[HandlerParameter(true, "25", Min = "10", Max = "100", Step = "1")]
		public int AroonPeriod
		{
			// Token: 0x0600008A RID: 138 RVA: 0x000039C9 File Offset: 0x00001BC9
			get;
			// Token: 0x0600008B RID: 139 RVA: 0x000039D1 File Offset: 0x00001BD1
			set;
		}

		// Token: 0x1700002D RID: 45
		[HandlerParameter(true, "50", Min = "10", Max = "100", Step = "10")]
		public int Filter
		{
			// Token: 0x0600008C RID: 140 RVA: 0x000039DA File Offset: 0x00001BDA
			get;
			// Token: 0x0600008D RID: 141 RVA: 0x000039E2 File Offset: 0x00001BE2
			set;
		}

		// Token: 0x1700002E RID: 46
		[HandlerParameter(true, "0", Min = "10", Max = "100", Step = "10", Name = "Output:\n0-osc,1-trend")]
		public int Output
		{
			// Token: 0x0600008E RID: 142 RVA: 0x000039EB File Offset: 0x00001BEB
			get;
			// Token: 0x0600008F RID: 143 RVA: 0x000039F3 File Offset: 0x00001BF3
			set;
		}
	}
}
