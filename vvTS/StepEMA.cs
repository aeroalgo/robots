using System;
using System.Collections.Generic;
using TSLab.Script.Handlers;
using TSLab.Script.Helpers;

namespace vvTSLtools
{
	// Token: 0x02000168 RID: 360
	[HandlerCategory("vvAverages"), HandlerName("Step EMA")]
	public class StepEMA : IDouble2DoubleHandler, IOneSourceHandler, IDoubleReturns, IStreamHandler, IHandler, IDoubleInputs, IContextUses
	{
		// Token: 0x06000B6C RID: 2924 RVA: 0x0002EC19 File Offset: 0x0002CE19
		public IList<double> Execute(IList<double> src)
		{
			return StepEMA.GenStepEMA(src, this.Context, this.EMAperiod, this.ERangePeriod, this.ERangeWindow);
		}

		// Token: 0x06000B6B RID: 2923 RVA: 0x0002E9AC File Offset: 0x0002CBAC
		public static IList<double> GenStepEMA(IList<double> src, IContext context, int _EMAperiod, int _ERangePeriod, int _ERangeWindow)
		{
			int count = src.Count;
			double[] array = new double[count];
			double[] array2 = new double[count];
			double[] array3 = new double[count];
			double[] array4 = new double[count];
			if (_ERangeWindow > _ERangePeriod)
			{
				_ERangeWindow = _ERangePeriod / 2;
			}
			double num = 2.0 / ((double)_EMAperiod + 1.0);
			double num2 = 2.0 / ((double)_ERangePeriod + 1.0);
			array[0] = src[0];
			for (int i = 1; i < count; i++)
			{
				double num3 = 0.0;
				double num4 = Indicators.Highest(src, i, _ERangeWindow);
				double num5 = Indicators.Lowest(src, i, _ERangeWindow);
				double num6 = num4 - num5;
				array2[i] = (1.0 - num2) * array2[i - 1] + num2 * num6;
				double num7 = array2[i];
				if (array3[i - 1] != 0.0)
				{
					num3 = (1.0 - num) * array3[i - 1] + num * src[i];
				}
				if (array4[i - 1] != 0.0)
				{
					num3 = (1.0 - num) * array4[i - 1] + num * src[i];
				}
				if (num3 == 0.0)
				{
					num3 = src[i];
				}
				if (num3 < src[i] - num7)
				{
					num3 = src[i] - num7;
				}
				if (num3 > src[i] + num7)
				{
					num3 = src[i] + num7;
				}
				double num8;
				if (array3[i - 1] == 0.0)
				{
					num8 = array4[i - 1];
				}
				else
				{
					num8 = array3[i - 1];
				}
				if (num3 > num8)
				{
					array3[i] = num3;
					array4[i] = 0.0;
					if (array3[i - 1] == 0.0)
					{
						array3[i - 1] = num8;
					}
				}
				else
				{
					array4[i] = num3;
					array3[i] = 0.0;
					if (array4[i - 1] == 0.0)
					{
						array4[i - 1] = num8;
					}
				}
			}
			for (int j = 0; j < count; j++)
			{
				array[j] = ((array3[j] == 0.0) ? array4[j] : array3[j]);
			}
			return array;
		}

		// Token: 0x170003C4 RID: 964
		public IContext Context
		{
			// Token: 0x06000B6D RID: 2925 RVA: 0x0002EC39 File Offset: 0x0002CE39
			get;
			// Token: 0x06000B6E RID: 2926 RVA: 0x0002EC41 File Offset: 0x0002CE41
			set;
		}

		// Token: 0x170003C1 RID: 961
		[HandlerParameter(true, "30", Min = "1", Max = "60", Step = "1")]
		public int EMAperiod
		{
			// Token: 0x06000B65 RID: 2917 RVA: 0x0002E976 File Offset: 0x0002CB76
			get;
			// Token: 0x06000B66 RID: 2918 RVA: 0x0002E97E File Offset: 0x0002CB7E
			set;
		}

		// Token: 0x170003C2 RID: 962
		[HandlerParameter(true, "14", Min = "1", Max = "30", Step = "1")]
		public int ERangePeriod
		{
			// Token: 0x06000B67 RID: 2919 RVA: 0x0002E987 File Offset: 0x0002CB87
			get;
			// Token: 0x06000B68 RID: 2920 RVA: 0x0002E98F File Offset: 0x0002CB8F
			set;
		}

		// Token: 0x170003C3 RID: 963
		[HandlerParameter(true, "8", Min = "1", Max = "20", Step = "1")]
		public int ERangeWindow
		{
			// Token: 0x06000B69 RID: 2921 RVA: 0x0002E998 File Offset: 0x0002CB98
			get;
			// Token: 0x06000B6A RID: 2922 RVA: 0x0002E9A0 File Offset: 0x0002CBA0
			set;
		}
	}
}
